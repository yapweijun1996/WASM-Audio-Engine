/** Version of the SDK-owned error-code contract. */
export const ERROR_CONTRACT_VERSION = 1 as const;
export type AudioEngineErrorCode =
  | "INVALID_ARGUMENT"
  | "INVALID_AUDIO"
  | "UNSUPPORTED_FORMAT"
  | "LIMIT_EXCEEDED"
  | "DSP_ERROR"
  | "INITIALIZATION_FAILED";

/** SDK-owned failure. Caller I/O, callback errors, and abort reasons propagate unchanged. */
export class AudioEngineError extends Error {
  readonly contractVersion = ERROR_CONTRACT_VERSION;
  constructor(
    readonly code: AudioEngineErrorCode,
    message: string,
    options?: ErrorOptions,
  ) {
    super(message, options);
    this.name = "AudioEngineError";
  }
}

/** Restrict translation to the actual WASM boundary, preserving the original cause. */
export function guardDsp<T>(operation: () => T): T {
  try {
    return operation();
  } catch (cause) {
    throw new AudioEngineError("DSP_ERROR", "Audio processing failed", {
      cause,
    });
  }
}
