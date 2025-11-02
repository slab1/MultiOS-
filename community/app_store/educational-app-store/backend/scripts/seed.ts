import { Pool } from 'pg';
import bcrypt from 'bcryptjs';
import dotenv from 'dotenv';

dotenv.config();

const pool = new Pool({
  host: process.env.DB_HOST || 'localhost',
  port: parseInt(process.env.DB_PORT || '5432'),
  database: process.env.DB_NAME || 'educational_app_store',
  user: process.env.DB_USER || 'postgres',
  password: process.env.DB_PASSWORD,
});

async function seed() {
  try {
    console.log('Seeding database with initial data...');
    
    // Create admin user
    const adminPassword = await bcrypt.hash('admin123', 10);
    await pool.query(`
      INSERT INTO users (email, password_hash, name, role, institution, email_verified)
      VALUES ($1, $2, $3, $4, $5, $6)
      ON CONFLICT (email) DO NOTHING
    `, ['admin@edustore.com', adminPassword, 'Admin User', 'admin', 'Educational App Store', true]);

    // Create sample developer
    const devPassword = await bcrypt.hash('dev123', 10);
    await pool.query(`
      INSERT INTO users (email, password_hash, name, role, institution, email_verified)
      VALUES ($1, $2, $3, $4, $5, $6)
      ON CONFLICT (email) DO NOTHING
    `, ['developer@edustore.com', devPassword, 'Sample Developer', 'developer', 'Tech Academy', true]);

    console.log('Database seeding completed!');
  } catch (error) {
    console.error('Seeding failed:', error);
    process.exit(1);
  } finally {
    await pool.end();
  }
}

seed();