CREATE TABLE join_roles (
  join_role_id bigserial PRIMARY KEY,
  guild_id bigint NOT NULL,
  role_id bigint NOT NULL,
  UNIQUE (guild_id, role_id),
  CONSTRAINT FK_guild_info FOREIGN KEY (guild_id)
    REFERENCES guild_info (guild_id)
    ON DELETE CASCADE
    ON UPDATE NO ACTION
);

