ALTER TABLE registrations ADD COLUMN reg_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE registrations DROP COLUMN contact_uri;
