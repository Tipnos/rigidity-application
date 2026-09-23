-- Accounts are Steam-only: drop email, password and email confirmation data
ALTER TABLE users
    DROP COLUMN email,
    DROP COLUMN hash,
    DROP COLUMN reset_password_hash,
    DROP COLUMN password_hash_expire_at,
    DROP COLUMN email_confirmation_required;
