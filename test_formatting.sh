#!/bin/bash

# Test script for LLM formatting
# This tests if llama-cli can run properly

LLAMA_CLI="/Users/ajithr/Desktop/attempt3/supavoice2/supavoice/src-tauri/resources/llama-cli"
MODEL_DIR="/Users/ajithr/Library/Application Support/com.supavoice.Supavoice/models"

echo "🔍 Checking for LLM models..."

# Check for Gemma model
if [ -f "$MODEL_DIR/gemma-2-2b-instruct" ]; then
    MODEL_PATH="$MODEL_DIR/gemma-2-2b-instruct"
    echo "✅ Found Gemma model"
elif [ -f "$MODEL_DIR/qwen2-1.5b-instruct" ]; then
    MODEL_PATH="$MODEL_DIR/qwen2-1.5b-instruct"
    echo "✅ Found Qwen model"
else
    echo "❌ No LLM model found!"
    echo "Please download a model from Settings first:"
    echo "  - Gemma 2 2B Instruct (1.71 GB)"
    echo "  - Qwen2 1.5B Instruct (986 MB)"
    exit 1
fi

echo ""
echo "🧪 Testing formatting with test transcript..."
echo ""

TEST_TRANSCRIPT="Hey I wanted to discuss the project timeline with you. Let's meet tomorrow at 3pm to go over the details and finalize everything."

PROMPT="<|im_start|>system
You are a helpful assistant that rewrites voice transcripts as professional emails.<|im_end|>
<|im_start|>user
Rewrite the following voice transcript as a professional email. Make it clear, concise, and well-structured with proper greeting and closing.

Transcript: ${TEST_TRANSCRIPT}<|im_end|>
<|im_start|>assistant
"

echo "📝 Test Transcript:"
echo "$TEST_TRANSCRIPT"
echo ""
echo "🤖 Running llama.cpp..."
echo ""

"$LLAMA_CLI" -m "$MODEL_PATH" -p "$PROMPT" -n 256 --temp 0.7 -ngl 99 --no-display-prompt 2>/dev/null

echo ""
echo ""
echo "✅ Test complete!"
