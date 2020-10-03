-- Add migration script here
CREATE TABLE guild_info (
  guild_id bigint PRIMARY KEY,
  prefix text NOT NULL
);

CREATE TABLE commands (
  command_id bigserial PRIMARY KEY,
  guild_id bigint NOT NULL,
  name text NOT NULL,
  content TEXT NOT NULL,
  CONSTRAINT FK_guild_info FOREIGN KEY (guild_id)
    REFERENCES guild_info (guild_id)
    ON DELETE CASCADE
    ON UPDATE NO ACTION
);

CREATE TABLE react_roles (
  react_role_id bigserial PRIMARY KEY,
  guild_id bigint NOT NULL,
  channel_id bigint NOT NULL,
  role_name text NOT NULL,
  reaction_emoji text NOT NULL,
  CONSTRAINT FK_guild_info FOREIGN KEY (guild_id)
    REFERENCES guild_info (guild_id)
    ON DELETE CASCADE
    ON UPDATE NO ACTION
);
