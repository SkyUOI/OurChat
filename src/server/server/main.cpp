#include <base/basedef.h>
#include <base/filesys.h>
#include <csignal>
#include <cstdlib>
#include <easylogging++.h>
#include <filesystem>
#include <gflags/gflags.h>
#include <server/server.h>
#ifdef UNITTEST
#include <gtest/gtest.h>
#endif

using asio::ip::tcp;

INITIALIZE_EASYLOGGINGPP

DEFINE_string(dbcfg, "../resource/database.json",
    "show the config file 's path to server");
DEFINE_bool(clear_log, false, "make server clear the log.");

static void quit_server(int sig) {
    ourchat::database::quit();
    LOG(INFO) << "quit the server";
    exit(EXIT_SUCCESS);
}

static std::string LogRootPath = "log";
static el::base::SubsecondPrecision LogSsPrec(3);

/**
 * @brief 设置日志
 */
static void ConfigureLogger() {
    std::filesystem::create_directories("log");
    // 日志器设置
    std::string datetimeY
        = el::base::utils::DateTime::getDateTime("%Y", &LogSsPrec);
    std::string datetimeYM
        = el::base::utils::DateTime::getDateTime("%Y%M", &LogSsPrec);
    std::string datetimeYMd
        = el::base::utils::DateTime::getDateTime("%Y%M%d", &LogSsPrec);
    std::string filePath
        = LogRootPath + "/" + datetimeY + "_" + datetimeYM + "_";
    std::string filename;
    el::Configurations defaultConf;
    defaultConf.setToDefault();
    // 建议使用setGlobally
    defaultConf.setGlobally(el::ConfigurationType::Enabled, "true");
    defaultConf.setGlobally(el::ConfigurationType::ToFile, "true");
    defaultConf.setGlobally(el::ConfigurationType::ToStandardOutput, "true");
    defaultConf.setGlobally(el::ConfigurationType::SubsecondPrecision, "6");
    defaultConf.setGlobally(el::ConfigurationType::PerformanceTracking, "true");
    defaultConf.setGlobally(el::ConfigurationType::LogFlushThreshold, "1");
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Global) + ".log";
    defaultConf.set(el::Level::Global, el::ConfigurationType::Filename,
        filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Debug) + ".log";
    defaultConf.set(
        el::Level::Debug, el::ConfigurationType::Filename, filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Error) + ".log";
    defaultConf.set(
        el::Level::Error, el::ConfigurationType::Filename, filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Fatal) + ".log";
    defaultConf.set(
        el::Level::Fatal, el::ConfigurationType::Filename, filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Info) + ".log";
    defaultConf.set(
        el::Level::Info, el::ConfigurationType::Filename, filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Trace) + ".log";
    defaultConf.set(
        el::Level::Trace, el::ConfigurationType::Filename, filePath + filename);
    filename = datetimeYMd + "_"
        + el::LevelHelper::convertToString(el::Level::Warning) + ".log";
    defaultConf.set(el::Level::Warning, el::ConfigurationType::Filename,
        filePath + filename);
    el::Loggers::reconfigureAllLoggers(defaultConf);
}

int main(int argc, char** argv) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    ConfigureLogger();
    if (FLAGS_clear_log) {
        fs::remove_all("log");
        LOG(INFO) << "clear the log";
        exit(EXIT_SUCCESS);
    }
    LOG(INFO) << "Ourchat server starting";
    signal(SIGINT, quit_server);
    ourchat::database::init(FLAGS_dbcfg);
    LOG(INFO) << "init the database";
    puts("Ourchat server is ready to work");
#ifdef UNITTEST
    // 启动单元测试
    testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
#else
    asio::io_context io_context;
    ourchat::server server(io_context);
    io_context.run();
#endif
    return 0;
}
