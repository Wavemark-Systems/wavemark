#!/usr/bin/env python3
"""
Basic usage example for Wavemark Python bindings.
"""

import wavemark_python

def main():
    print("Wavemark Python Example")
    print("======================")
    
    # Simple function call
    message = wavemark_python.hello_world()
    print(f"Message: {message}")

if __name__ == "__main__":
    main()