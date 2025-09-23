// SRP character count calculation utility
// Based on the payment tables in backend/src/data/srp.rs

// Daily payment amounts (in millions) mapped to character count
const DAILY_PAYMENTS = [
  { amount: 20, characters: 1 },
  { amount: 35, characters: 2 },
  { amount: 45, characters: 3 },
  { amount: 50, characters: 4 },
  { amount: 55, characters: 5 },
  { amount: 60, characters: 6 },
  { amount: 65, characters: 7 },
  { amount: 70, characters: 8 },
  { amount: 75, characters: 9 },
  { amount: 80, characters: 10 }
];

// Per focus payment amounts (in millions) mapped to character count
const PER_FOCUS_PAYMENTS = [
  { amount: 125, characters: 1 },
  { amount: 225, characters: 2 },
  { amount: 295, characters: 3 },
  { amount: 330, characters: 4 },
  { amount: 365, characters: 5 },
  { amount: 400, characters: 6 },
  { amount: 435, characters: 7 },
  { amount: 470, characters: 8 },
  { amount: 505, characters: 9 },
  { amount: 540, characters: 10 },
  { amount: 600, characters: 11 }
];

/**
 * Calculate the number of characters covered by an SRP payment
 * @param {number} paymentAmount - Payment amount in ISK (not millions)
 * @param {string} coverageType - Either "daily" or "per_focus"
 * @returns {number} Number of characters covered, or 0 if not a valid SRP payment
 */
export function getCharacterCount(paymentAmount, coverageType) {
  // Convert from ISK to millions for comparison
  const amountInMillions = paymentAmount / 1_000_000;
  
  // Use appropriate payment table based on coverage type
  const paymentTable = coverageType === "per_focus" ? PER_FOCUS_PAYMENTS : DAILY_PAYMENTS;
  
  // Find matching payment amount (with small tolerance for floating point precision)
  const matchingPayment = paymentTable.find(payment => 
    Math.abs(payment.amount - amountInMillions) < 0.01
  );
  
  return matchingPayment ? matchingPayment.characters : 0;
}

/**
 * Get character count display text
 * @param {number} paymentAmount - Payment amount in ISK
 * @param {string} coverageType - Either "daily" or "per_focus"
 * @returns {string} Display text for character count
 */
export function getCharacterCountText(paymentAmount, coverageType) {
  const count = getCharacterCount(paymentAmount, coverageType);
  
  if (count === 0) {
    return "Unknown";
  } else if (count === 1) {
    return "1 character";
  } else {
    return `${count} characters`;
  }
}
