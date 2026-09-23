-- Lossy rollback: dropped values can't be restored, so columns come back nullable
ALTER TABLE users
    ADD email VARCHAR(100) NULL UNIQUE,
    ADD hash VARCHAR(159) NULL,
    ADD reset_password_hash VARCHAR(159) NULL UNIQUE,
    ADD password_hash_expire_at TIMESTAMP NULL,
    ADD email_confirmation_required BOOLEAN NOT NULL DEFAULT FALSE;
