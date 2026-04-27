# Zectrix Cloud API 完整参考

## 认证

所有请求需携带 Header：
```
X-API-Key: zt_your_key_here
```

Base URL：
```
https://cloud.zectrix.com/open/v1
```

## 设备管理

### GET /devices — 获取设备列表

```bash
curl "https://cloud.zectrix.com/open/v1/devices" \
  -H "X-API-Key: zt_your_key_here"
```

**响应：**
```json
{
  "code": 0,
  "data": [
    {
      "deviceId": "AA:BB:CC:DD:EE:FF",
      "alias": "我的设备",
      "board": "bread-compact-wifi"
    }
  ]
}
```

| 字段 | 说明 |
|------|------|
| deviceId | 设备ID（MAC地址） |
| alias | 设备别名 |
| board | 设备型号/板型 |

---

## 待办事项

### GET /todos — 获取待办列表

查询参数：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| status | integer | 否 | 过滤状态：0=待完成, 1=已完成 |
| deviceId | string | 否 | 设备ID(MAC地址)，过滤指定设备的待办 |

```bash
curl "https://cloud.zectrix.com/open/v1/todos?status=0" \
  -H "X-API-Key: zt_your_key_here"
```

**响应字段：**

| 字段 | 说明 |
|------|------|
| id | 待办ID |
| title | 待办标题 |
| description | 待办描述 |
| dueDate | 截止日期 (yyyy-MM-dd) |
| dueTime | 截止时间 (HH:mm) |
| repeatType | 重复类型：none=不重复 |
| repeatWeekday | 重复星期（数字） |
| repeatMonth | 重复月份 |
| repeatDay | 重复日期 |
| status | 状态：0=待完成, 1=已完成 |
| priority | 优先级 |
| completed | 是否已完成 |
| deviceId | 关联设备ID |
| deviceName | 关联设备名称 |
| createDate | 创建时间 (yyyy-MM-dd HH:mm:ss) |
| updateDate | 更新时间戳 |

### POST /todos — 创建待办

```bash
curl "https://cloud.zectrix.com/open/v1/todos" \
  -H "X-API-Key: zt_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"title":"买牛奶","priority":1,"deviceId":"AA:BB:CC:DD:EE:FF"}'
```

**请求体字段：**
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| title | string | 是 | 待办标题 |
| description | string | 否 | 待办描述 |
| dueDate | string | 否 | 截止日期 |
| dueTime | string | 否 | 截止时间 |
| repeatType | string | 否 | 重复类型 |
| repeatWeekday | int | 否 | 重复星期 |
| repeatMonth | int | 否 | 重复月份 |
| repeatDay | int | 否 | 重复日期 |
| priority | int | 否 | 优先级 |
| deviceId | string | 否 | 关联设备ID |

### PUT /todos/{id} — 更新待办

```bash
curl -X PUT "https://cloud.zectrix.com/open/v1/todos/1" \
  -H "X-API-Key: zt_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"title":"买牛奶和面包"}'
```

### PUT /todos/{id}/complete — 切换完成状态

```bash
curl -X PUT "https://cloud.zectrix.com/open/v1/todos/1/complete" \
  -H "X-API-Key: zt_your_key_here"
```

### DELETE /todos/{id} — 删除待办

```bash
curl -X DELETE "https://cloud.zectrix.com/open/v1/todos/1" \
  -H "X-API-Key: zt_your_key_here"
```

---

## 显示推送

### POST /devices/{deviceId}/display/image — 推送图片

deviceId 为 MAC 地址格式。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| images | file[] | 是 | 图片文件，支持多张(最多5张)，单张不超过2MB |
| dither | boolean | 否 | 是否使用抖动算法(默认true)，关闭则使用硬阈值二值化 |
| pageId | integer | 否 | 页面编号(1-5)，指定后会持久化存储 |

```bash
curl "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/image" \
  -H "X-API-Key: zt_your_key_here" \
  -F "images=@photo1.png" \
  -F "images=@photo2.png" \
  -F "pageId=1" \
  -F "dither=true"
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "totalPages": 2,
    "pushedPages": 2,
    "pageId": "1"
  }
}
```

### POST /devices/{deviceId}/display/text — 推送纯文本

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| text | string | 是 | 文本内容(最多5000字)，支持换行 |
| fontSize | integer | 否 | 字体大小(12-48，默认20) |
| pageId | string | 否 | 页面编号(1-5)，指定后会持久化存储 |

```bash
curl "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/text" \
  -H "X-API-Key: zt_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"text":"今日待办：\n1. 买牛奶\n2. 写报告","fontSize":20,"pageId":"1"}'
```

### POST /devices/{deviceId}/display/structured-text — 推送标题+正文

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| title | string | 否 | 标题文本(最多200字)，与body至少填一项 |
| body | string | 否 | 正文内容(最多5000字)，支持换行 |
| pageId | string | 否 | 页面编号(1-5)，指定后会持久化存储 |

```bash
curl "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/structured-text" \
  -H "X-API-Key: zt_your_key_here" \
  -H "Content-Type: application/json" \
  -d '{"title":"今日计划","body":"1. 晨会 9:00\n2. 代码评审\n3. 发布上线"}'
```

### DELETE /devices/{deviceId}/display/pages/{pageId} — 删除页面

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| deviceId | string | 是 | 设备ID(MAC地址) |
| pageId | integer | 否 | 页面编号，不传则删除全部页面 |

```bash
# 删除指定页面
curl -X DELETE "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/pages/1" \
  -H "X-API-Key: zt_your_key_here"

# 清空全部页面（不传 pageId）
curl -X DELETE "https://cloud.zectrix.com/open/v1/devices/AA:BB:CC:DD:EE:FF/display/pages" \
  -H "X-API-Key: zt_your_key_here"
```

---

## 错误码

| code | 说明 |
|------|------|
| 0 | 成功 |
| 非0 | 错误，具体信息见 msg 字段 |
