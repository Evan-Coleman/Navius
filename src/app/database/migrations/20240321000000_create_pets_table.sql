-- Create pets table
CREATE TABLE IF NOT EXISTS pets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    species VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_pets_species ON pets(species);
CREATE INDEX IF NOT EXISTS idx_pets_created_at ON pets(created_at);

-- Add table and column documentation
COMMENT ON TABLE pets IS 'Stores information about pets in the system';
COMMENT ON COLUMN pets.id IS 'Unique identifier for the pet';
COMMENT ON COLUMN pets.name IS 'Name of the pet';
COMMENT ON COLUMN pets.species IS 'Species of the pet (e.g., Cat, Dog)';
COMMENT ON COLUMN pets.age IS 'Age of the pet in years';
COMMENT ON COLUMN pets.created_at IS 'Timestamp when the pet record was created';
COMMENT ON COLUMN pets.updated_at IS 'Timestamp when the pet record was last updated'; 