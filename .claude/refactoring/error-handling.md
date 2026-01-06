```

5. Transcriber Returns Error for Local Provider
transcriber.rs:113-118:

Provider::Local => {
    // Local provider is handled separately in the recording controller
    // This shouldn't be called for Local provider
    Err(TranscriptionError::ApiError(
        "Local provider uses ModelLoader, not Transcriber".to_string(),
    ))
}
This is a code smell. from_config returning an error for a valid provider breaks the abstraction. Consider a different pattern (e.g., enum dispatch or trait object that includes LocalClient).

```