use jiff::Timestamp;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{encode::IsNull, error::BoxDynError, Decode, Encode, Postgres, Type};

/// A PostgreSQL timestamp that can be either a concrete timestamp or infinity.
///
/// PostgreSQL supports special values 'infinity' and '-infinity' for timestamps,
/// which represent timestamps far in the future and past respectively. This type
/// allows working with these values in Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgTimestamp {
    Timestamp(Timestamp),
    Infinity,
    NegInfinity,
}

// PostgreSQL uses microseconds since 2000-01-01 00:00:00 UTC
// These are the special values for infinity
const PG_EPOCH_MICROS: i64 = 946_684_800_000_000; // 2000-01-01 in microseconds since Unix epoch
const INFINITY: i64 = i64::MAX;
const NEG_INFINITY: i64 = i64::MIN;

impl Type<Postgres> for PgTimestamp {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("TIMESTAMPTZ")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("TIMESTAMPTZ") || *ty == PgTypeInfo::with_name("TIMESTAMP")
    }
}

impl Encode<'_, Postgres> for PgTimestamp {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match self {
            PgTimestamp::Infinity => {
                buf.extend_from_slice(&INFINITY.to_be_bytes());
                Ok(IsNull::No)
            }
            PgTimestamp::NegInfinity => {
                buf.extend_from_slice(&NEG_INFINITY.to_be_bytes());
                Ok(IsNull::No)
            }
            PgTimestamp::Timestamp(ts) => {
                // Convert jiff::Timestamp to PostgreSQL microseconds
                let unix_micros = ts.as_microsecond();
                let pg_micros = unix_micros - PG_EPOCH_MICROS;
                buf.extend_from_slice(&pg_micros.to_be_bytes());
                Ok(IsNull::No)
            }
        }
    }
}

impl Decode<'_, Postgres> for PgTimestamp {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        if bytes.len() != 8 {
            return Err("invalid timestamp length".into());
        }

        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        let pg_micros = i64::from_be_bytes(buf);

        match pg_micros {
            INFINITY => Ok(PgTimestamp::Infinity),
            NEG_INFINITY => Ok(PgTimestamp::NegInfinity),
            micros => {
                // Convert PostgreSQL microseconds to Unix microseconds
                let unix_micros = micros + PG_EPOCH_MICROS;
                let ts = Timestamp::from_microsecond(unix_micros)?;
                Ok(PgTimestamp::Timestamp(ts))
            }
        }
    }
}

impl PgTimestamp {
    /// Returns true if this is the infinity value
    pub fn is_infinity(&self) -> bool {
        matches!(self, PgTimestamp::Infinity)
    }

    /// Returns true if this is the negative infinity value
    pub fn is_neg_infinity(&self) -> bool {
        matches!(self, PgTimestamp::NegInfinity)
    }

    /// Returns the concrete timestamp if this is not infinity
    pub fn timestamp(&self) -> Option<Timestamp> {
        match self {
            PgTimestamp::Timestamp(ts) => Some(*ts),
            _ => None,
        }
    }
}

impl From<Timestamp> for PgTimestamp {
    fn from(ts: Timestamp) -> Self {
        PgTimestamp::Timestamp(ts)
    }
}
