/**
 * Basic functionality tests for Wavemark TypeScript bindings.
 */

import { hello_world } from '../pkg/wavemark_typescript';

describe('Wavemark TypeScript Bindings', () => {
    test('should return hello world message', () => {
        const result = hello_world();
        expect(typeof result).toBe('string');
        expect(result).toContain('Wavemark');
        expect(result).toContain('TypeScript');
    });
});