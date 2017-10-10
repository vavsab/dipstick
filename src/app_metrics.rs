//! Static metrics are used to define metrics that share a single persistent metrics scope.
//! Because the scope never changes (it is "global"), all that needs to be provided by the
//! application is the metrics values.
//!
//! Compared to [ScopeMetrics], static metrics are easier to use and provide satisfactory metrics
//! in many applications.
//!
//! If multiple [AppMetrics] are defined, they'll each have their scope.

use core::*;
use std::sync::Arc;

// TODO define an 'AsValue' trait + impl for supported number types, then drop 'num' crate
pub use num::ToPrimitive;

/// Wrap the metrics backend to provide an application-friendly interface.
pub fn metrics<M, S>(sink: S) -> AppMetrics<M, S>
    where S: Sink<M> + 'static,
          M: 'static + Clone + Send + Sync
{
    let static_scope = sink.new_scope(true);
    AppMetrics {
        prefix: "".to_string(),
        scope: static_scope,
        sink: Arc::new(sink),
    }
}

/// A monotonic counter metric.
/// Since value is only ever increased by one, no value parameter is provided,
/// preventing programming errors.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Marker<M> {
    metric: M,
    #[derivative(Debug="ignore")]
    scope: ScopeFn<M>,
}

impl<M> Marker<M> {
    /// Record a single event occurence.
    pub fn mark(&self) {
        self.scope.as_ref()(Scope::Write(&self.metric, 1));
    }
}

/// A counter that sends values to the metrics backend
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Counter<M> {
    metric: M,
    #[derivative(Debug="ignore")]
    scope: ScopeFn<M>,
}

impl<M> Counter<M> {
    /// Record a value count.
    pub fn count<V>(&self, count: V) where V: ToPrimitive {
        self.scope.as_ref()(Scope::Write(&self.metric, count.to_u64().unwrap()));
    }
}

/// A gauge that sends values to the metrics backend
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Gauge<M> {
    metric: M,
    #[derivative(Debug="ignore")]
    scope: ScopeFn<M>,
}

impl<M> Gauge<M> {
    /// Record a value point for this gauge.
    pub fn value<V>(&self, value: V) where V: ToPrimitive {
        self.scope.as_ref()(Scope::Write(&self.metric, value.to_u64().unwrap()));
    }
}

/// A timer that sends values to the metrics backend
/// Timers can record time intervals in multiple ways :
/// - with the time! macrohich wraps an expression or block with start() and stop() calls.
/// - with the time(Fn) methodhich wraps a closure with start() and stop() calls.
/// - with start() and stop() methodsrapping around the operation to time
/// - with the interval_us() method, providing an externally determined microsecond interval
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Timer<M> {
    metric: M,
    #[derivative(Debug="ignore")]
    scope: ScopeFn<M>,
}

impl<M> Timer<M> {
    /// Record a microsecond interval for this timer
    /// Can be used in place of start()/stop() if an external time interval source is used
    pub fn interval_us<V>(&self, interval_us: V) -> V where V: ToPrimitive {
        self.scope.as_ref()(Scope::Write(&self.metric, interval_us.to_u64().unwrap()));
        interval_us
    }

    /// Obtain a opaque handle to the current time.
    /// The handle is passed back to the stop() method to record a time interval.
    /// This is actually a convenience method to the TimeHandle::now()
    /// Beware, handles obtained here are not bound to this specific timer instance
    /// _for now_ but might be in the future for safety.
    /// If you require safe multi-timer handles, get them through TimeType::now()
    pub fn start(&self) -> TimeHandle {
        TimeHandle::now()
    }

    /// Record the time elapsed since the start_time handle was obtained.
    /// This call can be performed multiple times using the same handle,
    /// reporting distinct time intervals each time.
    /// Returns the microsecond interval value that was recorded.
    pub fn stop(&self, start_time: TimeHandle) -> u64 {
        let elapsed_us = start_time.elapsed_us();
        self.interval_us(elapsed_us)
    }

    /// Record the time taken to execute the provided closure
    pub fn time<F, R>(&self, operations: F) -> R where F: FnOnce() -> R {
        let start_time = self.start();
        let value: R = operations();
        self.stop(start_time);
        value
    }
}

/// Variations of this should also provide control of the metric recording scope.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AppMetrics<M, S> {
    prefix: String,
    sink: Arc<S>,
    #[derivative(Debug="ignore")]
    scope: ScopeFn<M>,
}

impl <M, S> AppMetrics<M, S>
    where S: Sink<M>,
          M: Clone + Send + Sync
{
    fn qualified_name<AS>(&self, name: AS) -> String
        where AS: Into<String> + AsRef<str>
    {
        // FIXME is there a way to return <S> in both cases?
        if self.prefix.is_empty() {
            return name.into()
        }
        let mut buf:String = self.prefix.clone();
        buf.push_str(name.as_ref());
        buf.to_string()
    }

    /// Get an event counter of the provided name.
    pub fn marker<AS>(&self, name: AS) -> Marker<M>
        where AS: Into<String> + AsRef<str>, M: Send + Sync
    {
        let metric = self.sink.new_metric(Kind::Marker, &self.qualified_name(name), 1.0);
        Marker { metric, scope: self.scope.clone(), }
    }

    /// Get a counter of the provided name.
    pub fn counter<AS>(&self, name: AS) -> Counter<M>
        where AS: Into<String> + AsRef<str>, M: Send + Sync
    {
        let metric = self.sink.new_metric(Kind::Counter, &self.qualified_name(name), 1.0);
        Counter { metric, scope: self.scope.clone(), }
    }

    /// Get a timer of the provided name.
    pub fn timer<AS>(&self, name: AS) -> Timer<M>
        where AS: Into<String> + AsRef<str>, M: Send + Sync
    {
        let metric = self.sink.new_metric(Kind::Timer, &self.qualified_name(name), 1.0);
        Timer { metric, scope: self.scope.clone(), }
    }

    /// Get a gauge of the provided name.
    pub fn gauge<AS>(&self, name: AS) -> Gauge<M>
        where AS: Into<String> + AsRef<str>, M: Send + Sync
    {
        let metric = self.sink.new_metric(Kind::Gauge, &self.qualified_name(name), 1.0);
        Gauge { metric, scope: self.scope.clone(), }
    }

    /// Prepend the metrics name with a prefix.
    /// Does not affect metrics that were already obtained.
    pub fn with_prefix<IS>(&self, prefix: IS) -> Self where IS: Into<String> {
        AppMetrics {
            prefix: prefix.into(),
            sink: self.sink.clone(),
            scope: self.scope.clone(),
        }
    }

    /// Forcefully flush the backing metrics scope.
    /// This is usually not required since static metrics use auto flushing scopes.
    /// The effect, if any, of this method depends on the selected metrics backend.
    pub fn flush_scope(&mut self) {
        self.scope.as_ref()(Scope::Flush);
    }
}


#[cfg(feature = "bench")]
mod microbench {

    use ::*;
    use test;

    #[bench]
    fn time_bench_direct_dispatch_event(b: &mut test::Bencher) {
        let (sink, _source) = aggregate();
        let metrics = metrics(sink);
        let marker = metrics.marker("aaa");
        b.iter(|| test::black_box(marker.mark()));
    }

}