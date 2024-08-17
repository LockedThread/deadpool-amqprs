use std::convert::Infallible;

use amqprs::connection::OpenConnectionArguments;
use deadpool::Runtime;

use crate::{Manager, Pool, PoolBuilder, PoolConfig};

/// Possible methods of how a connection is recycled.
///
/// The default is [`Fast`] which does not check the connection health or
/// perform any verification.
///
/// [`Fast`]: RecyclingMethod::Fast
/// [`Verified`]: RecyclingMethod::Verified
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecyclingMethod {
    /// Only run [`Connection::is_open()`][1] when recycling existing connections.
    ///
    /// Unless you have special needs this is a safe choice.
    ///
    /// [1]: amqprs::connection::Connection::is_open
    Fast,

    /// Run [`Connection::is_open()`][1] and execute a test query.
    ///
    /// This is slower, but guarantees that the rabbitmq connection is ready to
    /// be used. Normally, [`Connection::is_open()`][1] should be enough to filter
    /// out bad connections, but under some circumstances (i.e. hard-closed
    /// network connections) it's possible that [`Connection::is_open()`][1]
    /// returns `false` while the connection is dead. You will receive an error
    /// on your first query then.
    ///
    /// [1]: amqprs::connection::Connection::is_open
    Verified,
}

impl Default for RecyclingMethod {
    fn default() -> Self {
        Self::Fast
    }
}

/// Configuration object.
///
/// # Example
///
/// ```rs
/// use deadpool_amqprs::Config;
/// use amqprs::connection::OpenConnectionArguments;
///
/// let config = Config::new_with_con_args(OpenConnectionArguments::default())
///
/// let pool = config.create_pool();
///
/// // Do things with `pool`.
/// ```
#[derive(Clone, Default)]
pub struct Config {
    /// The [`OpenConnectionArguments`] passed to [`amqprs::connection::Connection::open`].
    pub con_args: OpenConnectionArguments,
    /// The [`PoolConfig`] passed to deadpool.
    pub pool_config: Option<PoolConfig>,

    pub recycling_method: RecyclingMethod,
}

impl Config {
    /// Creates a new config with [`OpenConnectionArguments`] and optionally [`PoolConfig`].
    #[must_use]
    pub const fn new(
        con_args: OpenConnectionArguments,
        pool_config: Option<PoolConfig>,
        recycling_method: Option<RecyclingMethod>,
    ) -> Self {
        Self {
            con_args,
            pool_config,
            recycling_method: match recycling_method {
                Some(method) => method,
                None => RecyclingMethod::Fast,
            },
        }
    }

    /// Creates a new config with only [`OpenConnectionArguments`]
    #[must_use]
    pub const fn new_with_con_args(con_args: OpenConnectionArguments) -> Self {
        Self {
            con_args,
            pool_config: None,
            recycling_method: RecyclingMethod::Fast,
        }
    }

    /// Creates a new pool with the current config.
    ///
    /// # Info
    ///
    /// Unlike other `deadpool-*` libs, `deadpool-amqprs` does not require user to pass [`deadpool::Runtime`],
    /// because amqprs is built on top of `tokio`, meaning one can only use `tokio` with it.
    #[must_use]
    pub fn create_pool(&self) -> Pool {
        self.builder()
            .build()
            .expect("`PoolBuilder::build` errored when it shouldn't")
    }

    /// Returns a [`PoolBuilder`] using the current config.
    ///
    /// # Info
    ///
    /// Unlike other `deadpool-*` libs, `deadpool-amqprs` does not require user to pass [`deadpool::Runtime`],
    /// because amqprs is built on top of `tokio`, meaning one can only use `tokio` with it.
    pub fn builder(&self) -> PoolBuilder {
        Pool::builder(Manager::new(
            self.con_args.clone(),
            self.recycling_method.clone(),
        ))
        .config(self.pool_config.unwrap_or_default())
        .runtime(Runtime::Tokio1)
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("pool_config", &self.pool_config)
            .finish_non_exhaustive()
    }
}

pub type ConfigError = Infallible;
