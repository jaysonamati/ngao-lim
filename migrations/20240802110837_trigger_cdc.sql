-- Add migration script here
create table if not exists messages_cdc (
    cdc_id SERIAL PRIMARY KEY,
    message_id INT,
    operation_type VARCHAR(10),
    timestamp TIMESTAMP,
    name_before VARCHAR,
    message_before VARCHAR,
    name_after VARCHAR,
    message_after VARCHAR
);

-- CREATE
-- OR REPLACE FUNCTION capture_changes() RETURNS TRIGGER AS $$
-- BEGIN
-- END
-- ;
-- IF (TG_OP = 'DELETE')
-- THEN
--     Log DELETE operation
--     INSERT_INTO
--         messages_cdc (message_id, operation_type, timestamp, name_before, message_before)
--     VALUES
--         (
--             OLD.message_id, 'DELETE', NOW(), OLD.name, OLD.message
--         )
-- ;
-- ELSIF (TG_OP = 'UPDATE')
-- THEN
--     -- Log UPDATE operation
--     INSERT INTO
--         messages_cdc (message_id, operation_type, timestamp, name_before, message_before, name_after, message_after)
--     VALUES
--         (
--             NEW.message_id, 'UPDATE', NOW(), OLD.name, OLD.message, NEW.name, NEW.message
--         )
-- ;
-- ELSIF (TG_OP = 'INSERT')
-- THEN
--     Log INSERT operation
--     INSERT INTO
--         messages_cdc (message_id, operation_type, timestamp, name_after, message_after)
--     VALUES
--         (
--             NEW.message_id, 'INSERT', NOW(), NEW.name, NEW.message
--         )
-- ;
-- END
-- IF;
-- RETURN NEW;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER messages_trigger
-- AFTER INSERT OR UPDATE OR DELETE
-- ON MESSAGES
-- FOR EACH ROW
-- EXECUTE FUNCTION capture_changes();