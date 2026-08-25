# OCR confidence threshold

The implementation accepts an explicit confidence threshold and falls back to image pixels when the OCR result is below it. No default threshold is selected because the feature contract leaves this product decision open; callers must provide the policy rather than inheriting a guessed value.
