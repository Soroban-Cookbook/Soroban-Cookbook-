#!/bin/bash

# Verification script for Feedback System
# Run: bash verify-feedback-system.sh

echo "=== Feedback System Verification ==="
echo ""

# Check directory structure
echo "1. Checking directory structure..."
DIRS=("forms" "surveys" "reviews" "actions" "communication")
for dir in "${DIRS[@]}"; do
    if [ -d "../$dir" ]; then
        echo "   ✓ $dir directory exists"
    else
        echo "   ✗ $dir directory missing"
    fi
done

# Check required files
echo ""
echo "2. Checking required files..."
FILES=(
    "../README.md"
    "../forms/feedback-form-template.md"
    "../surveys/config.yml"
    "../reviews/checklist.md"
    "../actions/tracking-template.md"
    "../communication/response-templates.md"
)
for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $(basename $file) exists"
    else
        echo "   ✗ $(basename $file) missing"
    fi
done

# Validate YAML syntax (if yq is available)
echo ""
echo "3. Validating configurations..."
if command -v yq &> /dev/null; then
    if yq eval '../surveys/config.yml' &> /dev/null; then
        echo "   ✓ surveys/config.yml is valid YAML"
    else
        echo "   ✗ surveys/config.yml has invalid YAML"
    fi
else
    echo "   ⚠ yq not installed, skipping YAML validation"
fi

# Check for acceptance criteria
echo ""
echo "4. Acceptance Criteria Verification:"
CRITERIA=("Feedback forms" "Survey tools" "Review process" "Action tracking" "Communication")
for criterion in "${CRITERIA[@]}"; do
    echo "   [ ] $criterion"
done

echo ""
echo "=== Verification Complete ==="
echo ""
echo "Manual verification required for acceptance criteria."
echo "Update phase file when all criteria are met."