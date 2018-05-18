//! Publicly exposed metric macros are defined here.
//! Although `dipstick` does not have a macro-based API,
//! in some situations they can make instrumented code simpler.

// TODO add #[timer("name")] method annotation processors

/// A convenience macro to wrap a block or an expression with a start / stop timer.
/// Elapsed time is sent to the supplied statsd client after the computation has been performed.
/// Expression result (if any) is transparently returned.
#[macro_export]
macro_rules! time {
    ($timer: expr, $body: expr) => {{
        let start_time = $timer.start();
        let value = $body;
        $timer.stop(start_time);
        value
    }};
}

/// Metrics can be used from anywhere (public), does not need to declare metrics in this block.
#[macro_export]
macro_rules! metrics {
    // TYPED
    // typed, public, no metrics
    (<$METRIC_TYPE:ty> pub $METRIC_ID:ident = $e:expr $(;)*) => {
        lazy_static! { pub static ref $METRIC_ID: MetricScope<$METRIC_TYPE> = $e.into(); }
    };
    // typed, public, some metrics
    (<$METRIC_TYPE:ty> pub $METRIC_ID:ident = $e:expr => { $($REMAINING:tt)+ }) => {
        lazy_static! { pub static ref $METRIC_ID: MetricScope<$METRIC_TYPE> = $e.into(); }
        __metrics_block!($METRIC_ID; $($REMAINING)*);
    };
    // typed, module, no metrics
    (<$METRIC_TYPE:ty> $METRIC_ID:ident = $e:expr $(;)*) => {
        lazy_static! { static ref $METRIC_ID: MetricScope<$METRIC_TYPE> = $e.into(); }
    };
    // typed, module, some metrics
    (<$METRIC_TYPE:ty> $METRIC_ID:ident = $e:expr => { $($REMAINING:tt)+ }) => {
        lazy_static! { static ref $METRIC_ID: MetricScope<$METRIC_TYPE> = $e.into(); }
        __metrics_block!($METRIC_ID; $($REMAINING)*);
    };
    // typed, reuse predeclared
    (<$METRIC_TYPE:ty> $METRIC_ID:ident => { $($REMAINING:tt)+ }) => {
        __metrics_block!($METRIC_ID; $($REMAINING)*);
    };
    // typed, unidentified, some metrics
    (<$METRIC_TYPE:ty> $e:expr => { $($REMAINING:tt)+ }) => {
        lazy_static! { static ref UNIDENT_METRIC: MetricScope<$METRIC_TYPE> = $e.into(); }
        __metrics_block!(UNIDENT_METRIC; $($REMAINING)*);
    };
    // typed, root, some metrics
    (<$METRIC_TYPE:ty> { $($REMAINING:tt)+ }) => {
        lazy_static! { static ref ROOT_METRICS: MetricScope<$METRIC_TYPE> = ().into(); }
        __metrics_block!(ROOT_METRICS; $($REMAINING)*);
    };

    (pub $METRIC_ID:ident = $e:expr $(;)*) => {
        metrics! {<Dispatch> pub $METRIC_ID = $e; }
    };
    (pub $METRIC_ID:ident = $e:expr => { $($REMAINING:tt)+ }) => {
        metrics! {<Dispatch> pub $METRIC_ID = $e => { $($REMAINING)* } }
    };
    ($METRIC_ID:ident = $e:expr $(;)*) => {
        metrics! {<Dispatch> $METRIC_ID = $e; }
    };
    ($METRIC_ID:ident = $e:expr => { $($REMAINING:tt)+ }) => {
        metrics! {<Dispatch> $METRIC_ID = $e => { $($REMAINING)* } }
    };
    ($METRIC_ID:ident => { $($REMAINING:tt)+ }) => {
        metrics! {<Dispatch> $METRIC_ID => { $($REMAINING)* } }
    };
    ($e:expr => { $($REMAINING:tt)+ }) => {
        metrics! {<Dispatch> $e => { $($REMAINING)* } }
    };

}

/// Internal macro required to abstract over pub/non-pub versions of the macro
#[macro_export]
#[doc(hidden)]
macro_rules! __metrics_block {
    ($SCOPE:ident;
    $(#[$attr:meta])* pub Counter $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* pub static ref $METRIC_ID:
            Counter = $SCOPE.counter($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* Counter $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* static ref $METRIC_ID:
            Counter = $SCOPE.counter($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* pub Marker $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* pub static ref $METRIC_ID:
            Marker = $SCOPE.marker($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* Marker $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* static ref $METRIC_ID:
            Marker = $SCOPE.marker($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* pub Gauge $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* pub static ref $METRIC_ID:
            Gauge = $SCOPE.gauge($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* Gauge $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* static ref $METRIC_ID:
            Gauge = $SCOPE.gauge($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* pub Timer $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* pub static ref $METRIC_ID:
            Timer = $SCOPE.timer($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($SCOPE:ident;
    $(#[$attr:meta])* Timer $METRIC_ID:ident : $METRIC_NAME:expr; $($REMAINING:tt)*) => {
        lazy_static! { $(#[$attr])* static ref $METRIC_ID:
            Timer = $SCOPE.timer($METRIC_NAME); }
        __metrics_block!($SCOPE; $($REMAINING)*);
    };
    ($METRIC_ID:ident;) => ()
}

/// Define application-scoped metrics.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! app_metrics {
    ($type_param: ty, $metric_id: ident = ($($SCOPE: expr),+ $(,)*)) => {
        lazy_static! {
            pub static ref $metric_id: MetricScope<$type_param> = metric_scope(($($SCOPE),*));
        }
    };
    ($type_param: ty, $metric_id: ident = [$($SCOPE: expr),+ $(,)*]) => {
        lazy_static! {
            pub static ref $metric_id: MetricScope<$type_param> = metric_scope(&[$($SCOPE),*][..],);
        }
    };
    ($type_param: ty, $metric_id: ident = $SCOPE: expr) => {
        lazy_static! {
            pub static ref $metric_id: MetricScope<$type_param> = $SCOPE.into();
        }
    };
}

/// Define application-scoped markers.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! app_marker {
    (<$type_param: ty> $SCOPE: expr => { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! {
            $(pub static ref $metric_id: Marker = $SCOPE.marker( $m_exp );)*
        }
     };
}

/// Define application-scoped counters.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! app_counter {
    (<$type_param: ty> $SCOPE: expr => { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! {
            $(pub static ref $metric_id: Counter = $SCOPE.counter( $m_exp );)*
        }
    };
}

/// Define application-scoped gauges.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! app_gauge {
    (<$type_param: ty> $SCOPE: expr => { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! {
            $(pub static ref $metric_id: Gauge = $SCOPE.gauge( $m_exp );)*
        }
    };
}

/// Define application-scoped timers.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! app_timer {
    (<$type_param: ty> $SCOPE: expr => { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! {
            $(pub static ref $metric_id: Timer = $SCOPE.timer( $m_exp );)*
        }
    };
}

/////////////
// MOD SCOPE

/// Define module-scoped metrics.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! mod_metrics {
    ($type_param: ty, $metric_id: ident = ($($SCOPE: expr),+ $(,)*)) => {
        lazy_static! {
            static ref $metric_id: MetricScope<$type_param> = metric_scope(($($SCOPE),*));
        }
    };
    ($type_param: ty, $metric_id: ident = [$($SCOPE: expr),+ $(,)*]) => {
        lazy_static! { static ref $metric_id: MetricScope<$type_param> =
            metric_scope(&[$($SCOPE),*][..],); }
    };
    ($type_param: ty, $metric_id: ident = $mod_metrics: expr) => {
        lazy_static! { static ref $metric_id: MetricScope<$type_param> =
            $mod_metrics.into(); }
    };
}

/// Define module-scoped markers.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! mod_marker {
    ($type_param: ty, $mod_metrics: expr, { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! { $(static ref $metric_id: Marker =
            $mod_metrics.marker( $m_exp );)* }
    };
}

/// Define module-scoped counters.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! mod_counter {
    ($type_param: ty, $mod_metrics: expr, { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! { $(static ref $metric_id: Counter =
            $mod_metrics.counter( $m_exp );)* }
    };
}

/// Define module-scoped gauges.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! mod_gauge {
    ($type_param: ty, $mod_metrics: expr, { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! { $(static ref $metric_id: Gauge =
            $mod_metrics.gauge( $m_exp );)* }
    };
    ($type_param: ty, $mod_metrics: expr, $metric_id: ident: $m_exp: expr) => {
        lazy_static! { static ref $metric_id: Gauge =
            $mod_metrics.gauge( $m_exp ); }
    }
}

/// Define module-scoped timers.
#[macro_export]
#[deprecated(since = "0.7.0", note = "Use metrics!() instead")]
macro_rules! mod_timer {
    ($type_param: ty, $mod_metrics: expr, { $($metric_id: ident: $m_exp: expr),* $(,)* } ) => {
        lazy_static! { $(static ref $metric_id: Timer =
            $mod_metrics.timer( $m_exp );)* }
    };
}

#[cfg(test)]
mod test {
    use self_metrics::*;

    metrics!(<Aggregate> DIPSTICK_METRICS.with_suffix("test_prefix") => {
        Marker M1: "failed";
        Marker M2: "success";
        Counter C1: "failed";
        Counter C2: "success";
        Gauge G1: "failed";
        Gauge G2: "success";
        Timer T1: "failed";
        Timer T2: "success";
    });

    #[test]
    fn call_new_macro_defined_metrics() {
        M1.mark();
        M2.mark();

        C1.count(1);
        C2.count(2);

        G1.value(1);
        G2.value(2);

        T1.interval_us(1);
        T2.interval_us(2);
    }
}

#[cfg(test)]
mod legacy_test {
    use self_metrics::*;

    metrics!(<Aggregate> TEST_METRICS = DIPSTICK_METRICS.with_suffix("test_prefix"));

    app_marker!(<Aggregate> TEST_METRICS => {
        M1: "failed",
        M2: "success",
    });

    app_counter!(<Aggregate> TEST_METRICS => {
        C1: "failed",
        C2: "success",
    });

    app_gauge!(<Aggregate> TEST_METRICS => {
        G1: "failed",
        G2: "success",
    });

    app_timer!(<Aggregate> TEST_METRICS => {
        T1: "failed",
        T2: "success",
    });

    #[test]
    fn call_old_macro_defined_metrics() {
        M1.mark();
        M2.mark();

        C1.count(1);
        C2.count(2);

        G1.value(1);
        G2.value(2);

        T1.interval_us(1);
        T2.interval_us(2);
    }
}
