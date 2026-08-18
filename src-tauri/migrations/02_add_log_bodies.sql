-- 日志表增加请求/响应体字段，用于前端详情查看

ALTER TABLE request_logs ADD COLUMN request_body TEXT;
ALTER TABLE request_logs ADD COLUMN response_body TEXT;
