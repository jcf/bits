CREATE USER bits WITH PASSWORD 'please' CREATEDB;
CREATE USER datomic WITH PASSWORD 'datomic';

CREATE DATABASE bits_dev OWNER bits;
CREATE DATABASE bits_test OWNER bits;
CREATE DATABASE datomic OWNER datomic;

GRANT pg_signal_backend TO bits;

\c bits_dev
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS postgis;

\c bits_test
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS postgis;

\c datomic
CREATE TABLE IF NOT EXISTS datomic_kvs (
  id text NOT NULL,
  rev integer,
  map text,
  val bytea,
  CONSTRAINT pk_id PRIMARY KEY (id)
);
ALTER TABLE datomic_kvs OWNER TO datomic;
GRANT ALL ON TABLE datomic_kvs TO datomic;
