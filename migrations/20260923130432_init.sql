CREATE DOMAIN uint2 AS int4
    CHECK (VALUE >= 0 AND VALUE < 65536);

CREATE TYPE enum_archetypes AS ENUM ('leader', 'spiker', 'healer', 'assassin');
CREATE TYPE enum_game_modes AS ENUM ('deathmatch', 'king_of_the_hill');
CREATE TYPE enum_maps AS ENUM ('ascent', 'inferno', 'colosseum', 'heaven', 'play_ground');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    nickname VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    steam_id TEXT NOT NULL UNIQUE,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    birth_date TIMESTAMP NOT NULL
);

CREATE TABLE custom_rooms (
    id SERIAL PRIMARY KEY,
    label VARCHAR(100) NOT NULL,
    user_id INT UNIQUE NOT NULL,
    nb_teams uint2 NOT NULL,
    max_player_per_team uint2 NOT NULL,
    current_game_mode enum_game_modes NOT NULL DEFAULT 'king_of_the_hill',
    current_map enum_maps NOT NULL DEFAULT 'inferno',
    matchmaking_ticket UUID NULL,

    CONSTRAINT fk_owner
        FOREIGN KEY (user_id)
            REFERENCES users (id)
            ON DELETE CASCADE
);

CREATE TABLE custom_room_slots (
    id SERIAL PRIMARY KEY,
    custom_room_id INT NOT NULL,
    team uint2 NOT NULL,
    team_position uint2 NOT NULL,
    user_id INT UNIQUE NOT NULL,
    current_archetype enum_archetypes NOT NULL,

    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
            REFERENCES users (id)
            ON DELETE CASCADE,

    CONSTRAINT fk_custom_room
        FOREIGN KEY (custom_room_id)
            REFERENCES custom_rooms (id)
            ON DELETE CASCADE
);
