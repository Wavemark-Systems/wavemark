/**
 * Basic usage example for Wavemark TypeScript bindings.
 */

import { hello_world } from '../pkg/wavemark_typescript';

function main() {
    console.log('Wavemark TypeScript Example');
    console.log('===========================');
    
    // Simple function call
    const message = hello_world();
    console.log(`Message: ${message}`);
}

// Run the example
main();