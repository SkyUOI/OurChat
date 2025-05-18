# OurChat 🚀

[![codecov](https://codecov.io/github/SkyUOI/OurChat/graph/badge.svg?token=U6BWN74URE)](https://codecov.io/github/SkyUOI/OurChat)[![License](https://img.shields.io/github/license/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/blob/main/LICENSE)[![GitHub stars](https://img.shields.io/github/stars/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/stargazers)[![GitHub issues](https://img.shields.io/github/issues/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/issues)[![GitHub pull requests](https://img.shields.io/github/issues-pr/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/pulls)[![GitHub release](https://img.shields.io/github/v/release/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/releases)[![Last Commit](https://img.shields.io/github/last-commit/skyuoi/ourchat)](https://github.com/skyuoi/ourchat/commits)

<!-- markdownlint-disable MD033 -->
<p align="center">
    <img src="./resource/logo.png" alt="OurChat_logo" />
</p>
<!-- markdownlint-enable MD033 -->

## [中文](./README-zh.md)

## 🌟 Introduction

OurChat is a chat application for Linux, Windows and macOS. It supports all platforms through Flutter.

⚠️ The project is under rapid development, and there is also a lot of work to be done. It cannot be used directly by now.

In the past year of 2024, we have achieved many outstanding tasks, and it is delightful that developers have devoted
so much passion to this project in their spare time.
Every developer deserves gratitude. 💖

## 📱 Feature Highlights

- 💬 Real-time messaging
- 👥 Group chats
- 🔒 End-to-end encryption
- 🌍 Cross-platform support
- 🚀 High performance, low latency
- 🛠️ Self-hostable

## Official Server

Server Address: `skyuoi.org:7777`. If you want to develop the client, you can also use it as your development server to work with. The docker image version it uses is `nightly` (It will be updated regularly but not nightly).

## 🚀 Vision & Plan

Provides a lightweight chat software that can easily run on devices like Raspberry Pi, allowing you to set up your own
chat server for your company, family, etc. At the same time, it has the potential to scale up to a high-performance
server capable of accommodating millions of users.

🔑 **Core Principles**:

- ✅ **Freedom & Openness**: Freedom and openness are the principles of our design, and you will experience much more freedom than other chat software
- 🔒 **Security**: End-to-end encryption and other security guarantees make OurChat a service you can trust
- 🛡️ **Privacy**: We absolutely protect your privacy!

## 🚀 Quick Start

### ⚠️ Security Notice

If you want to use it in the product environment, you should do a series of improvements, such as changing
the password of database. More information please refer to document.

### 🖥️ Server Deployment

```shell
cd docker
docker compose up -d
```

For More deployment methods, please refer
to [deployment document](https://ourchat.readthedocs.io/en/latest/docs/deploy/server-deploy.html)

## 🛠️ Build from source

Refer to [Build Document](https://ourchat.readthedocs.io/en/latest/docs/run/build.html)

## 📚 Documentation

Refer to [Documentation](https://ourchat.readthedocs.io/en/latest/), we deploy it to ReadTheDocs

## 🤝 Contribution

Please see [CONTRIBUTING](https://ourchat.readthedocs.io/en/latest/docs/development/contributing.html)

## 🌐 Community

- [Matrix](https://matrix.to/#/#skyuoiourchat:matrix.org)

## 📦 Supported Platforms

| Platform | Status                                                                                                 |
|:---------|:-------------------------------------------------------------------------------------------------------|
| Linux    | ![Linux Test](https://img.shields.io/github/actions/workflow/status/skyuoi/ourchat/rust_linux.yml)     |
| Windows  | ![Windows Test](https://img.shields.io/github/actions/workflow/status/skyuoi/ourchat/rust_windows.yml) |
| Macos    | ![Macos Test](https://img.shields.io/github/actions/workflow/status/skyuoi/ourchat/rust_macos.yml)     |
