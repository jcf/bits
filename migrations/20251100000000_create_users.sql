create extension if not exists btree_gist;

create table users (
  id bigserial primary key,
  password_hash text not null,
  created_at timestamptz not null default now()
);

create table email_addresses (
  id bigserial primary key,
  user_id bigint not null references users(id) on delete cascade,
  address text not null,
  valid_from timestamptz not null default now(),
  valid_to timestamptz not null default 'infinity',

  exclude using gist (
    address with =,
    tstzrange(valid_from, valid_to) with &&
  )
);

create index idx_email_addresses_by_address
on email_addresses(address)
where valid_to = 'infinity';

create index idx_email_addresses_by_user
on email_addresses(user_id)
where valid_to = 'infinity';

create table email_verifications (
  email_address_id bigint primary key references email_addresses(id) on delete cascade,
  verified_at timestamptz not null default now()
);

create table user_preferred_email (
  id bigserial primary key,
  user_id bigint not null references users(id) on delete cascade,
  email_address_id bigint not null references email_addresses(id) on delete cascade,
  valid_from timestamptz not null default now(),
  valid_to timestamptz not null default 'infinity',

  exclude using gist (
    user_id with =,
    tstzrange(valid_from, valid_to) with &&
  )
);

create view logins as
select
  u.id as user_id,
  u.password_hash,
  a.address as email
from users u
join email_addresses a on a.user_id = u.id
join email_verifications v on v.email_address_id = a.id
where a.valid_to = 'infinity';

create view preferred_email_addresses as
select
  u.id as user_id,
  a.address,
  v.verified_at
from users u
join user_preferred_email upe on upe.user_id = u.id
join email_addresses a on a.id = upe.email_address_id
left join email_verifications v on v.email_address_id = a.id
where upe.valid_to = 'infinity'
  and a.valid_to = 'infinity';

create view active_email_addresses as
select
  u.id as user_id,
  a.id as email_address_id,
  a.address,
  v.verified_at,
  upe.id is not null as preferred
from users u
join email_addresses a on a.user_id = u.id
left join email_verifications v on v.email_address_id = a.id
left join user_preferred_email upe on upe.email_address_id = a.id
  and upe.valid_to = 'infinity'
where a.valid_to = 'infinity';
