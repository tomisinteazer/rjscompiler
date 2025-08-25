// Complex JavaScript test for analyzer
function outerFunction(x) {
    let outerVar = x + 1;
    
    function innerFunction(y) {
        let innerVar = y + outerVar; // closure capture
        
        function deeplyNested() {
            return outerVar + innerVar; // multiple closure captures
        }
        
        return deeplyNested;
    }
    
    // This will make the scope unsafe
    function riskyFunction() {
        eval('let dynamicVar = 42;'); // eval usage
        return dynamicVar;
    }
    
    return {
        inner: innerFunction,
        risky: riskyFunction
    };
}

// Export for module test
export const config = { debug: true };
export function processData(data) {
    return data.map(x => x * 2);
}

// Arrow function with this binding
const arrowObj = {
    value: 42,
    getValue: () => this.value, // lexical this
    getValueRegular: function() { return this.value; } // dynamic this
};

// Block scoped variables
if (true) {
    let blockVar = 'scoped';
    const blockConst = 'also scoped';
    var functionScoped = 'function scoped';
}

console.log('Complex analysis test complete');