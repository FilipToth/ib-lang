use ibc::analysis::binding::symbols::VariableSymbol;
use ibc::analysis::binding::types::TypeKind;

fn symbol(identifier: &str) -> VariableSymbol {
    VariableSymbol {
        identifier: identifier.to_string(),
        var_type: TypeKind::Int,
        symbol_id: 1,
    }
}

#[test]
fn an_ordinary_identifier_is_its_own_name() {
    assert_eq!(symbol("COUNT").name(), "COUNT");
    assert_eq!(symbol("my_var").name(), "my_var");
}

#[test]
fn a_mangled_parameter_reports_the_users_name() {
    let cases = [
        ("$param$Stack<Int>$push$item", "item"),
        ("$param$Array<Int>$get$index", "index"),
        // '$' separates the parts, so '_' inside a name is not a problem
        ("$param$Array<Int>$get$my_index", "my_index"),
        // nor is a nested generic in the type
        ("$param$Collection<Array<Int>>$addItem$item", "item"),
    ];

    for (identifier, expected) in cases {
        assert_eq!(symbol(identifier).name(), expected, "{}", identifier);
    }
}
