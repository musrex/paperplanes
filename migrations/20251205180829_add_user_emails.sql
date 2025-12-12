-- Add migration script here

ALTER TABLE users
ADD COLUMN email TEXT;

UPDATE users 
SET email = 'leandro.n.lopez@protonmail.com'
WHERE email IS NULL;

ALTER TABLE users
ALTER COLUMN email SET NOT NULL;

