-- Public UUIDs: `id` / `*_id` become UUIDs exposed by the API, the SERIAL keys
-- stay internal as `pk` / `*_pk`. FK constraints and UNIQUE indexes follow the
-- column renames, existing rows are backfilled by the default.

ALTER TABLE users RENAME COLUMN id TO pk;
ALTER TABLE users ADD COLUMN id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid();

ALTER TABLE custom_rooms RENAME COLUMN id TO pk;
ALTER TABLE custom_rooms RENAME COLUMN user_id TO user_pk;
ALTER TABLE custom_rooms ADD COLUMN id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid();

ALTER TABLE custom_room_slots RENAME COLUMN id TO pk;
ALTER TABLE custom_room_slots RENAME COLUMN user_id TO user_pk;
ALTER TABLE custom_room_slots RENAME COLUMN custom_room_id TO custom_room_pk;
ALTER TABLE custom_room_slots ADD COLUMN id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid();
