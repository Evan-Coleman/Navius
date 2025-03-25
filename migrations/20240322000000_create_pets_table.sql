-- Create the pets table
CREATE TABLE IF NOT EXISTS pets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    pet_type VARCHAR(50) NOT NULL,
    breed VARCHAR(100),
    age INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add an index on the name field for faster lookups
CREATE INDEX IF NOT EXISTS idx_pets_name ON pets(name);

-- Add a trigger to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_pets_updated_at
    BEFORE UPDATE ON pets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 