# Online Judge 后端

## 运行项目
- 安装`rust`
- 输入`cargo run --release`运行

## 前后端接口

后端默认地址为``http://127.0.0.1:8001``，前后端通过`http`请求收发`json`进行通信。

前端请求格式：

```json5
{
    "data" {
        "id": 1
        // ...
    }
}
```

后端发送格式：
```json5
{
    "status": "success", // or "error"
    "message": "login successfully", // 额外说明的信息，一般success留空，error具体说明
    "data": {
        "id": 1
        // ...
    }
}
```

具体的请求信息和返回信息见`module`目录下的文件内的注释