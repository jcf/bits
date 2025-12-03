create table if not exists sessions (
  id varchar(128) not null primary key,
  expires bigint,
  session text not null,
  user_id bigint references users(id) on delete cascade
);
