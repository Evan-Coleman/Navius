-- Create pets table
CREATE TABLE IF NOT EXISTS pets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    pet_type VARCHAR(255) NOT NULL,
    breed VARCHAR(255),
    age INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_pets_pet_type ON pets(pet_type);
CREATE INDEX IF NOT EXISTS idx_pets_name ON pets(name); 