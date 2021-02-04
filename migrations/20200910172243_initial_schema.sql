CREATE TABLE guild_info (
  guild_id bigint PRIMARY KEY,
  prefix text NOT NULL
);

CREATE TABLE commands (
  command_id bigserial PRIMARY KEY,
  guild_id bigint NOT NULL,
  name TEXT NOT NULL,
  content TEXT NOT NULL,
  UNIQUE (guild_id, name),
  CONSTRAINT FK_guild_info FOREIGN KEY (guild_id)
    REFERENCES guild_info (guild_id)
    ON DELETE CASCADE
    ON UPDATE NO ACTION
);

CREATE TABLE react_roles (
  react_role_id bigserial PRIMARY KEY,
  guild_id bigint NOT NULL,
  channel_id bigint NOT NULL,
  message_id bigint NOT NULL,
  role_id bigint NOT NULL,
  reaction_emoji TEXT NOT NULL,
  UNIQUE (guild_id, channel_id, message_id, reaction_emoji),
  CONSTRAINT FK_guild_info FOREIGN KEY (guild_id)
    REFERENCES guild_info (guild_id)
    ON DELETE CASCADE
    ON UPDATE NO ACTION
);
