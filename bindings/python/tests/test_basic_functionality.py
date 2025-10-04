"""
Basic functionality tests for Wavemark Python bindings.
"""

import wavemark_python

def test_hello_world():
    """Test the hello_world function."""
    result = wavemark_python.hello_world()
    assert isinstance(result, str)
    assert "Wavemark" in result
    assert "Python" in result