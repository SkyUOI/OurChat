set_project("OurChat")
set_version("0.1")

set_languages("c++20", "c17")
set_targetdir("bin")

add_includedirs("src/server/include")
add_requires("gtest")
add_requires("asio")
add_requires("easyloggingpp")
add_requires("jsoncpp")
add_requires("hash-library")
add_requires("gflags")

add_rules("mode.debug", "mode.release")
add_rules("plugin.compile_commands.autoupdate", {outputdir = ".vscode"})
add_rules("plugin.compile_commands.autoupdate")
if is_os("linux") then
    add_includedirs("/usr/include/mysql")
end

target("ourchat_server")
    set_kind("binary")
    add_files("src/**.cpp")
    add_packages("asio")
    add_packages("easyloggingpp")
    add_packages("jsoncpp")
    add_packages("hash-library")
    add_packages("gflags")
    if is_os("linux") then
        add_links("mysqlclient");
    end
    if is_os("windows") then
        add_links("libmysql");
    end
--target_link_libraries(OurChat_server ws2_32 wsock32)
--
--target_link_libraries(OurChat_server pthread)

target("unittest")
    set_kind("binary")
    set_default(false)
    add_defines("UNITTEST")
    add_files("src/server/**.cpp", "tests/server/unittest/**.cpp")
    add_packages("gtest")
    add_packages("asio")
    add_packages("easyloggingpp")
    add_packages("jsoncpp")
    add_packages("hash-library")
    add_packages("gflags")
    if is_os("linux") then
        add_links("mysqlclient");
    end
    if is_os("windows") then
        add_links("libmysql");
    end
