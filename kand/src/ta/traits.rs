use crate::KandError;
#[cfg(feature = "arrow")]
use arrow::record_batch::RecordBatch;

/// A trait for stateful technical indicators.
pub trait Indicator {
    /// The input type for a single update.
    type Input;
    /// The output type for a single update.
    type Output;

    /// Processes the next input value and updates internal state.
    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError>;

    /// Returns the current state as an Arrow RecordBatch for persistence.
    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<RecordBatch, KandError>;

    /// Restores the indicator state from an Arrow RecordBatch.
    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(&mut self, batch: &RecordBatch) -> Result<(), KandError>;
}

/// A trait for vectorized batch technical indicators.
///
/// Operates on N independent streams simultaneously.
pub trait BatchIndicator {
    /// The input type (usually an Arrow Array or slice of values).
    type Input;
    /// The output type (usually an Arrow Array).
    type Output;

    /// Processes the next batch of input values for all streams.
    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError>;

    /// Returns the entire batch state as an Arrow RecordBatch.
    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<RecordBatch, KandError>;

    /// Restores the entire batch state from an Arrow RecordBatch.
    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(&mut self, batch: &RecordBatch) -> Result<(), KandError>;
}

/// A trait defining the core logic for a technical indicator.
///
/// This can be used to generate stateful and batch variants automatically.
pub trait IndicatorLogic {
    type Input;
    type Output;
    type State: Clone;

    /// Initializes the state for a single stream.
    fn init_state(&self) -> Self::State;

    /// Processes a single input and updates state.
    fn next(&self, state: &mut Self::State, input: Self::Input) -> Self::Output;
}
