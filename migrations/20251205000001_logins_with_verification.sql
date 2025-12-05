create or replace view logins as
select
  u.id as user_id,
  u.password_hash,
  a.address as email,
  exists(
    select 1 from email_verifications ev
    where ev.email_address_id = a.id
  ) as verified
from users u
join email_addresses a on a.user_id = u.id
where a.valid_to = 'infinity';
