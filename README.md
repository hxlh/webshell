# webshell

`webshell` 是一个基于 Rust 开发的 Web Shell 工具，允许用户通过浏览器访问远程服务器的命令行。

## 特性

*   通过 WebSocket 进行实时交互式 Shell 会话。
*   支持自定义监听地址和端口。
*   支持会话超时设置。
*   需要用户名和密码进行认证。
*   提供静态 HTML 页面作为客户端。

## 构建

项目使用 Rust 语言编写，需要安装 Rust 工具链。

```bash
./build.sh
```

构建产物将位于 `dist/` 目录下，包括可执行文件 `webshell` 和静态资源 `web/shell.html`。

## 使用方法

启动 `webshell` 服务：

```bash
./dist/webshell run <your_server_ip>:<port> <timeout seconds>
```

参数说明：

*   `addr`: SSH 服务器的地址和端口，例如 `127.0.0.1:22`。
*   `timeout`: 会话超时时间（秒）。

启动后，程序会提示输入用于 SSH 连接的用户名和密码。

成功启动后，会输出访问地址，例如：

```text
INFO serve on http://<local_ip>:<assigned_port>/static/shell.html
```

在浏览器中打开此地址即可使用 Web Shell。

## 许可证

本项目使用 [LICENSE](LICENSE) 文件中指定的许可证。