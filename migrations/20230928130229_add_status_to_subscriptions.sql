-- Add migration script here
-- subscriptionテーブルにstatusカラムを追加
ALTER TABLE subscriptions ADD COLUMN status TEXT NULL;
