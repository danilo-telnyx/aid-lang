-- Seed data for webhook classifier demo
-- Real-world webhook examples from common services

INSERT OR IGNORE INTO webhooks (id, source, event_type, payload, classification, priority, confidence, created_at) VALUES
(1, 'github', 'push', '{"ref":"refs/heads/main","commits":[{"message":"fix: resolve null pointer in auth middleware"}],"repository":{"full_name":"acme/api-service"}}', 'deployment', 'medium', 0.91, '2026-02-20 08:15:00'),
(2, 'stripe', 'payment.failed', '{"type":"payment_intent.payment_failed","data":{"object":{"amount":4999,"currency":"usd","last_payment_error":{"message":"Card declined"}}}}', 'billing', 'high', 0.95, '2026-02-20 08:22:00'),
(3, 'datadog', 'alert.triggered', '{"title":"High CPU Usage on web-server-03","type":"metric alert","body":"CPU usage exceeded 90% for 5 minutes","priority":"P2"}', 'monitoring', 'high', 0.88, '2026-02-20 08:30:00'),
(4, 'pagerduty', 'incident.trigger', '{"event":{"routing_key":"svc-database","event_action":"trigger","payload":{"summary":"Database primary is unreachable","severity":"critical"}}}', 'urgent', 'critical', 0.97, '2026-02-20 08:35:00'),
(5, 'github', 'pull_request.opened', '{"action":"opened","pull_request":{"title":"feat: add webhook classifier endpoint","user":{"login":"dsmaldone"},"base":{"ref":"main"}}}', 'notification', 'low', 0.82, '2026-02-20 09:00:00'),
(6, 'sentry', 'error.created', '{"event_id":"abc123","project":"api-gateway","title":"NullPointerException in PaymentService.process","level":"error","url":"https://sentry.io/issues/12345"}', 'technical', 'high', 0.90, '2026-02-20 09:15:00'),
(7, 'stripe', 'invoice.created', '{"type":"invoice.created","data":{"object":{"amount_due":29900,"currency":"usd","customer":"cus_ABC123","status":"open"}}}', 'billing', 'medium', 0.93, '2026-02-20 09:30:00'),
(8, 'aws', 'cloudwatch.alarm', '{"AlarmName":"prod-memory-usage","NewStateValue":"ALARM","NewStateReason":"Threshold crossed: 1 datapoint (87.5) > 85.0","Region":"us-east-1"}', 'monitoring', 'high', 0.89, '2026-02-20 09:45:00'),
(9, 'github', 'deployment.created', '{"action":"created","deployment":{"environment":"production","ref":"v2.4.1","task":"deploy","description":"Production release v2.4.1"}}', 'deployment', 'medium', 0.94, '2026-02-20 10:00:00'),
(10, 'unknown', 'custom', '{"message":"FREE OFFER! You have been selected for an exclusive deal. Click here immediately!","from":"promo@spam-domain.xyz"}', 'spam', 'low', 0.96, '2026-02-20 10:15:00'),
(11, 'cloudflare', 'security.event', '{"action":"challenge","rule_id":"waf-sqli-001","description":"SQL injection attempt detected on /api/search","source_ip":"185.220.101.42"}', 'security', 'critical', 0.98, '2026-02-20 10:30:00'),
(12, 'slack', 'message.posted', '{"channel":"#engineering","user":"alice","text":"Deployed v2.4.1 to staging, running smoke tests now","ts":"1708420200"}', 'notification', 'low', 0.80, '2026-02-20 10:45:00');

-- Seed classification log
INSERT OR IGNORE INTO classification_log (id, webhook_id, new_classification, changed_by, reason, created_at) VALUES
(1, 4, 'urgent', 'reason:classify_webhook', 'Matched constraint: outage/down/critical keywords', '2026-02-20 08:35:01'),
(2, 11, 'security', 'reason:classify_webhook', 'Matched constraint: vulnerability/breach/unauthorized keywords', '2026-02-20 10:30:01');
