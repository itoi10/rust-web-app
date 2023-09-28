-- Add migration script here
-- BEGINでトランザクション開始。失敗した場合はロールバックする。
BEGIN;
    -- statusがNULLの場合はconfirmedに変更し、今後NULLを許容しないようにする。
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
