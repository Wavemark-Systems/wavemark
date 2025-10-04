"""
Comprehensive tests for Wavemark Python bindings.
These tests verify that the Python bindings work correctly.
"""

import pytest
import wavemark
import sys
import platform


class TestBasicFunctionality:
    """Test basic functionality of the wavemark module."""
    
    def test_module_import(self):
        """Test that the wavemark module can be imported."""
        assert wavemark is not None
        assert hasattr(wavemark, 'hello_world')
    
    def test_hello_world_function(self):
        """Test the hello_world function returns expected output."""
        result = wavemark.hello_world()
        
        # Verify return type
        assert isinstance(result, str)
        
        # Verify content
        assert "Wavemark" in result
        assert "Python" in result
        assert "bindings" in result
        
        # Verify it's not empty
        assert len(result) > 0
    
    def test_hello_world_consistency(self):
        """Test that hello_world returns consistent results."""
        result1 = wavemark.hello_world()
        result2 = wavemark.hello_world()
        
        assert result1 == result2
    
    def test_function_callable(self):
        """Test that hello_world is callable."""
        assert callable(wavemark.hello_world)


class TestModuleStructure:
    """Test the structure and attributes of the wavemark module."""
    
    def test_module_has_expected_attributes(self):
        """Test that the module has expected attributes."""
        expected_attrs = ['hello_world']
        
        for attr in expected_attrs:
            assert hasattr(wavemark, attr), f"Module missing attribute: {attr}"
    
    def test_module_docstring(self):
        """Test that the module has a docstring."""
        assert wavemark.__doc__ is not None
        assert len(wavemark.__doc__.strip()) > 0


class TestPlatformCompatibility:
    """Test platform-specific functionality."""
    
    def test_platform_info(self):
        """Test that we can get platform information."""
        # This tests that the module works on the current platform
        result = wavemark.hello_world()
        assert isinstance(result, str)
        
        # Log platform info for debugging
        print(f"Platform: {platform.platform()}")
        print(f"Python version: {sys.version}")
        print(f"Architecture: {platform.architecture()}")
    
    def test_unicode_support(self):
        """Test that the module handles Unicode correctly."""
        result = wavemark.hello_world()
        
        # Test that result can be encoded/decoded
        encoded = result.encode('utf-8')
        decoded = encoded.decode('utf-8')
        assert decoded == result


class TestErrorHandling:
    """Test error handling and edge cases."""
    
    def test_no_arguments_required(self):
        """Test that hello_world requires no arguments."""
        # Should not raise any exceptions
        result = wavemark.hello_world()
        assert isinstance(result, str)
    
    def test_function_signature(self):
        """Test the function signature."""
        import inspect
        
        sig = inspect.signature(wavemark.hello_world)
        # Should have no parameters
        assert len(sig.parameters) == 0


class TestPerformance:
    """Test basic performance characteristics."""
    
    def test_function_performance(self):
        """Test that the function executes in reasonable time."""
        import time
        
        start_time = time.time()
        result = wavemark.hello_world()
        end_time = time.time()
        
        execution_time = end_time - start_time
        
        # Should execute quickly (less than 1 second)
        assert execution_time < 1.0
        assert isinstance(result, str)


class TestIntegration:
    """Test integration with Python ecosystem."""
    
    def test_pickle_compatibility(self):
        """Test that results can be pickled/unpickled."""
        import pickle
        
        result = wavemark.hello_world()
        
        # Should be able to pickle the result
        pickled = pickle.dumps(result)
        unpickled = pickle.loads(pickled)
        
        assert unpickled == result
    
    def test_json_serialization(self):
        """Test that results can be JSON serialized."""
        import json
        
        result = wavemark.hello_world()
        
        # Should be able to serialize to JSON
        json_str = json.dumps(result)
        deserialized = json.loads(json_str)
        
        assert deserialized == result


if __name__ == "__main__":
    # Run tests if executed directly
    pytest.main([__file__, "-v"])
