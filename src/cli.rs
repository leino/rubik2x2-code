use ::ui;
use std::collections::HashMap;
use cube;
use std;

struct SideConfiguration
{
    pub configuration: [[cube::Side; 4]; 6]
}

pub struct Input
{
    pub aliases: ui::SideAliases,
    pub initial_cube: cube::Cube,
}

pub enum ArgumentReadingError
{
    InvalidAliasArguments{side_alias_error: SideAliasReadingError},
    InvalidCubeConfiguration{configuration_error: SideConfigurationError},
}

pub enum SideConfigurationError
{
    InvalidSide{argument: String},
    MissingOpeningCurlyBracket,
    MissingClosingCurlyBracket,
    InvalidSideAlias{expected_alias: String},
}

pub enum SideAliasReadingError
{
    TooFewAliasesSpecified{specified_alias_count: usize},
    TooManyEqualsSigns{argument: String},
    AmbiguousAlias{alias: String, side: cube::Side},
    InvalidSideSpecifier{argument: String},
    MultipleAliasesForSameSide{side: cube::Side},
}

impl SideAliasReadingError
{
    fn message(&self) -> String
    {
        use self::SideAliasReadingError::*;
        match self
        {
            &TooFewAliasesSpecified{specified_alias_count} => 
                format!("Only {} side aliases specified: 6 aliases are required (one for each side)",
                        specified_alias_count),
            &TooManyEqualsSigns{ref argument} =>
                format!("Invalid argument: {}: more than one equal sign", argument),
            &AmbiguousAlias{ref alias, side} =>
                format!("Ambiguous alias: {}: this alias also refers to '{}'",
                        alias, cube::Side::serialization(side)),
            &InvalidSideSpecifier{ref argument} =>
            {
                let mut valid_specifiers = String::new();
                for side_idx in 0..6
                {
                    let side = cube::Side::from(side_idx);
                    let serialization = cube::Side::serialization(side);
                    let separator = if side_idx < 5 {", "} else {""};
                    valid_specifiers = format!("{}{}{}", valid_specifiers, serialization, separator);
                }                
                format!(
                    "Invalid argument: {}: invalid side specifier\n\
                     Valid side specifiers are: {}\n",
                    argument,
                    valid_specifiers
                )
            },
            &MultipleAliasesForSameSide{side} =>
                format!("Multiple aliases for the same side: {}", cube::Side::serialization(side)),
        }
    }
}

impl ArgumentReadingError
{
    pub fn message(&self) -> String
    {
        use self::ArgumentReadingError::*;
        match self
        {
            &InvalidAliasArguments{ref side_alias_error} =>
                format!("Invalid alias arguments: {}", side_alias_error.message()),
            &InvalidCubeConfiguration{ref configuration_error} =>
                format!("Invalid cube configuration: {}", configuration_error.message()),
        }
    }
}

impl SideConfigurationError
{
    fn message(&self) -> String
    {
        use self::SideConfigurationError::*;
        match self
        {
            &InvalidSide{ref argument} =>
            {
                let mut allowed_sides = String::new();
                for side_idx in 0..6 {
                    let serialization = cube::Side::serialization(cube::Side::from(side_idx));
                    let separator = if side_idx < 5 {", "} else {""};
                    allowed_sides = format!("{}'{}'{}\n", allowed_sides, serialization, separator);
                }

                format!("Expected [side] in side configuration specification: {}\n\
                         Here, [side] may be one of the following: {}\n", argument, allowed_sides)
            },
            &MissingOpeningCurlyBracket =>
                format!("Invalid configuration: expected '{{'\n"),
            &MissingClosingCurlyBracket =>
                format!("Invalid configuration: last alias should be followed by a '}}'"),
            &InvalidSideAlias{ref expected_alias} => 
                format!("Invalid configuration: invalid alias: '{}'", expected_alias),
        }
    }
}

impl SideConfiguration {
    pub fn to_cube(&self) -> cube::Cube {
        let configurations: &[[cube::Side; 4]; 6] = &self.configuration;
        let mut transforms = [[[cube::Transform::identity(); 2]; 2]; 2];
        for side_idx in 0..6 {
            for entry_idx in 0..4 {
                let d0 = side_idx/2;
                let d1 = [1, 0, 0][d0];
                let d2 = [2, 2, 1][d0];
                let i0 = side_idx & 1;
                let (i1, i2) = ((entry_idx >> 0) & 1, (entry_idx >> 1) & 1);
                let mut index = [0; 3];
                index[d0] = i0;
                index[d1] = i1;
                index[d2] = i2;
                let side = cube::Side::from(side_idx as i32);
                let solved_side = configurations[side_idx][entry_idx];
                let side_normal = cube::normal(side);
                let solved_side_normal = cube::normal(solved_side);
                let column_idx = {
                    let mut idx = 0;
                    while idx < 3 {
                        if solved_side_normal[idx] != 0 {
                            break;
                        }
                        assert!(solved_side_normal[idx] == 0);
                        idx += 1;
                    }
                    assert!(idx < 3);
                    idx
                };
                for row_idx in 0..3 {
                    transforms[index[0]][index[1]][index[2]].entries[row_idx][column_idx] =
                        solved_side_normal[column_idx]*side_normal[row_idx];
                }
            }
        }

        cube::Cube {transforms: transforms}
    }
    
}

pub fn try_read_arguments<I>(
    argument_iterator: &mut I
) -> Result<Input, ArgumentReadingError>
where
    I: Iterator<Item = String>
{
    use self::ArgumentReadingError::*;
    let side_aliases =
    {
        match try_read_side_alias_arguments(argument_iterator)
        {
            Err(error) => return Err(InvalidAliasArguments{side_alias_error: error}),
            Ok(side_aliases) => side_aliases
        }
    };
    
    let configuration =
        {
            match try_read_side_configuration_arguments(&side_aliases, argument_iterator)
            {
                Err(error) => return Err(InvalidCubeConfiguration{configuration_error: error}),
                Ok(configuration) => configuration,
            }
        };

    return Ok(Input{aliases: side_aliases, initial_cube: configuration.to_cube()});
}

fn try_read_side_alias_arguments<I>(argument_iterator: &mut I) -> Result<ui::SideAliases, SideAliasReadingError>
where
    I: Iterator<Item = String>
{
    use self::SideAliasReadingError::*;
    let mut side_aliases: HashMap<String, cube::Side> = HashMap::new();
    
    while side_aliases.len() < 6 {
        let argument = {
            match argument_iterator.next() {
                None => return Err(TooFewAliasesSpecified{specified_alias_count: side_aliases.len()}),
                Some(argument) => argument,
            }
        };
        let (alias, side) = {
            let mut pieces = [""; 2];
            let mut piece_count = 0;
            for piece in argument.split("=") {
                if piece_count >= 2 {
                    return Err(TooManyEqualsSigns{argument: argument.clone()})
                }
                pieces[piece_count] = piece;
                piece_count += 1;
            }
            let alias: &str = pieces[0];
            if let Some(&side) = side_aliases.get(alias) {
                return
                    Err(AmbiguousAlias{alias: String::from(alias), side: side});
            }
            let side = {
                match cube::Side::deserialize(pieces[1]) {
                    None => {
                        return Err(InvalidSideSpecifier{argument: argument.clone()});
                    },
                    Some(side) => side,
                }
            };
            if side_aliases.values().any(|&s| s == side)
            {
                return Err(MultipleAliasesForSameSide{side});
            }
            (alias, side)
        };
        side_aliases.insert(String::from(alias), side);
    }

    return Ok(ui::SideAliases{aliases: side_aliases});
}

fn try_read_side_configuration_arguments<I>(
    side_aliases: &ui::SideAliases,
    argument_iterator: &mut I
) -> Result<SideConfiguration, SideConfigurationError>
where
    I: Iterator<Item = String>
{
    use self::SideConfigurationError::*;
    let mut side_configurations: [[cube::Side; 4]; 6] = [[cube::Side::L; 4]; 6];
    
    while let Some(argument) = argument_iterator.next() {
        let mut rest = argument.as_str();
        let side = {
            let mut try_read_side = || {
                for side_idx in 0..6 {
                    let side = cube::Side::from(side_idx);
                    let serialization = cube::Side::serialization(side);
                    let serialization_length = serialization.len();
                    if rest[0..serialization_length] == *serialization {
                        rest = &rest[serialization_length..];
                        return Ok(side);
                    }
                }
                return Err(InvalidSide{argument: argument.clone()});
            };
            
            match try_read_side()
            {
                Ok(side) => side,
                Err(message) => return Err(message),
            }
        };

        if let Some(0) = rest.find('{') {
            rest = &rest[1..];
        } else {
            return Err(MissingOpeningCurlyBracket);
        }
        
        let configuration = {

            let mut configuration: [cube::Side; 4] = unsafe { std::mem::uninitialized() };
            for i in 0..4 {
                let c = if i == 3 {'}'} else {','};
                let optional_c_idx = rest.find(c);
                match optional_c_idx {
                    None => {
                        return Err(MissingClosingCurlyBracket);
                    },
                    Some(c_idx) => {
                        let expected_alias = &rest[0..c_idx];
                        match side_aliases.optional_side(expected_alias) {
                            None => {
                                return Err(InvalidSideAlias{expected_alias: String::from(expected_alias)});
                            },
                            Some(side) => {
                                configuration[i] = *side;
                                rest = &rest[c_idx+1..];
                            }
                        }
                    }
                }
            }

            configuration
        };
        side_configurations[side as usize] = configuration.clone();
    }

    return Ok(SideConfiguration{configuration: side_configurations});
}
