# Online Judge 后端

## 运行项目
- 安装`rust`
- 输入`cargo run --release`运行

## 前后端接口

后端默认地址为``http://127.0.0.1:8001``，前后端通过`http`请求收发`json`进行通信。

前端请求格式：

```json
{
    data: { // 传输的数据
        ...
    }
}
```

后端发送格式：
```json
{
    status: "success" // or "error"
    message: ... // 额外说明的信息
    data: {
        ... // 传输的数据
    }
}
```