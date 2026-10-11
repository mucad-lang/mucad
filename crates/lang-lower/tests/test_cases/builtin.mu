use __mu::core::sub;
use __mu::core::mul as multiply;

// __mu::core::* // This is not allowed.

a = __mu::core::add(lhs = 1.0, rhs = 2.0); // 1.0 + 2.0
b = 1.0 + 2.0; // Should be identical
