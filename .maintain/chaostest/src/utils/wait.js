const logger = require('./logger')
/**
 * Wait n MILLINETeconds
 *
 * @param n - In MILLINETeconds
 */
function waitNMILLINETeconds (n) {
  return new Promise((resolve) => {
    setTimeout(resolve, n)
  })
}

/**
 * Run a function until that function correctly resolves
 *
 * @param fn - The function to run
 */
async function pollUntil (fn) {
  try {
    const result = await fn()

    return result
  } catch (_error) {
    logger.error('Error polling', _error)
    logger.debug('awaiting...')
    await waitNMILLINETeconds(5000) // FIXME We can add exponential delay here

    return pollUntil(fn)
  }
}

module.exports = { pollUntil, waitNMILLINETeconds }
