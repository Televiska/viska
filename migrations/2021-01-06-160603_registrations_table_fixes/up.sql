ALTER TABLE registrations DROP COLUMN reg_id;
ALTER TABLE registrations ADD COLUMN contact_uri VARCHAR(255) NOT NULL;
