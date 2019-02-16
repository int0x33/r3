use {
ExpandedName , QualName
} ; use interface :: {
Attribute , TreeSink , Quirks , AppendNode , create_element
} ; use tree_builder :: types :: * ; use tree_builder :: tag_sets :: * ; use tree_builder :: actions :: {
NoPush , Push , TreeBuilderActions , html_elem
} ; use tokenizer :: {
EndTag , StartTag , Tag
} ; use tokenizer :: states :: {
Rcdata , Rawtext , ScriptData , Plaintext
} ; use util :: str :: is_ascii_whitespace ; use std :: ascii :: AsciiExt ; use std :: mem :: replace ; use std :: borrow :: Cow :: Borrowed ; use std :: borrow :: ToOwned ; use tendril :: {
StrTendril , SliceExt
} ; fn any_not_whitespace ( x : & StrTendril ) -> bool {
x . chars ( ) . any ( | c | ! is_ascii_whitespace ( c ) )
} pub trait TreeBuilderStep < Handle > {
fn step ( & mut self , mode : InsertionMode , token : Token ) -> ProcessResult < Handle > ; fn step_foreign ( & mut self , token : Token ) -> ProcessResult < Handle > ;
} fn current_node < Handle > ( open_elems : & [ Handle ] ) -> & Handle {
open_elems . last ( ) . expect ( "no current element" )
} # [ doc ( hidden ) ] impl < Handle , Sink > TreeBuilderStep < Handle > for super :: TreeBuilder < Handle , Sink > where Handle : Clone , Sink : TreeSink < Handle = Handle > , {
fn step ( & mut self , mode : InsertionMode , token : Token ) -> ProcessResult < Handle > {
self . debug_step ( mode , & token ) ; match mode {
Initial => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => Done , CommentToken ( text ) => self . append_comment_to_doc ( text ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
if ! self . opts . iframe_srcdoc {
self . unexpected ( & token ) ; self . set_quirks_mode ( Quirks ) ;
} Reprocess ( BeforeHtml , token )
}
}
}
} , BeforeHtml => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => Done , CommentToken ( text ) => self . append_comment_to_doc ( text ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => {
self . create_root ( tag . attrs ) ; self . mode = BeforeHead ; Done
} last_arm_token => {
let enable_wildcards = match last_arm_token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "head" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => false , _ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => self . unexpected ( & tag ) , ( _ , token ) => {
self . create_root ( vec ! ( ) ) ; Reprocess ( BeforeHead , token )
}
}
}
} , BeforeHead => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => Done , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) => {
self . head_elem = Some ( self . insert_element_for ( tag ) ) ; self . mode = InHead ; Done
} last_arm_token => {
let enable_wildcards = match last_arm_token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "head" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => false , _ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => self . unexpected ( & tag ) , ( _ , token ) => {
self . head_elem = Some ( self . insert_phantom ( local_name ! ( "head" ) ) ) ; Reprocess ( InHead , token )
}
}
}
} , InHead => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "base" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "basefont" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "bgsound" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "link" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) => {
self . insert_and_pop_element_for ( tag ) ; DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "title" ) , ..
} ) => {
self . parse_raw_data ( tag , Rcdata )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noscript" ) , ..
} ) => {
if ( ! self . opts . scripting_enabled ) && ( tag . name == local_name ! ( "noscript" ) ) {
self . insert_element_for ( tag ) ; self . mode = InHeadNoscript ; Done
} else {
self . parse_raw_data ( tag , Rawtext )
}
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) => {
let elem = create_element ( & mut self . sink , QualName :: new ( None , ns ! ( html ) , local_name ! ( "script" ) ) , tag . attrs ) ; if self . is_fragment ( ) {
self . sink . mark_script_already_started ( & elem ) ;
} self . insert_appropriately ( AppendNode ( elem . clone ( ) ) , None ) ; self . open_elems . push ( elem ) ; self . to_raw_text_mode ( ScriptData )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "head" ) , ..
} ) => {
self . pop ( ) ; self . mode = AfterHead ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) => {
self . insert_element_for ( tag ) ; self . active_formatting . push ( Marker ) ; self . frameset_ok = false ; self . mode = InTemplate ; self . template_modes . push ( InTemplate ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => {
if ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
self . unexpected ( & tag ) ;
} else {
self . generate_implied_end ( thorough_implied_end ) ; self . expect_to_close ( local_name ! ( "template" ) ) ; self . clear_active_formatting_to_marker ( ) ; self . template_modes . pop ( ) ; self . mode = self . reset_insertion_mode ( ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => false , _ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => self . unexpected ( & tag ) , ( _ , token ) => {
self . pop ( ) ; Reprocess ( AfterHead , token )
}
}
}
} , InHeadNoscript => match token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "noscript" ) , ..
} ) => {
self . pop ( ) ; self . mode = InHead ; Done
} CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => self . step ( InHead , token ) , CommentToken ( _ ) => self . step ( InHead , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "basefont" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "bgsound" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "link" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) => self . step ( InHead , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noscript" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => false , _ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => self . unexpected ( & tag ) , ( _ , token ) => {
self . unexpected ( & token ) ; self . pop ( ) ; Reprocess ( InHead , token )
}
}
}
} , AfterHead => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "body" ) , ..
} ) => {
self . insert_element_for ( tag ) ; self . frameset_ok = false ; self . mode = InBody ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "frameset" ) , ..
} ) => {
self . insert_element_for ( tag ) ; self . mode = InFrameset ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "base" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "basefont" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "bgsound" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "link" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "title" ) , ..
} ) => {
self . unexpected ( & token ) ; let head = self . head_elem . as_ref ( ) . expect ( "no head element" ) . clone ( ) ; self . push ( & head ) ; let result = self . step ( InHead , token ) ; self . remove_from_stack ( & head ) ; result
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => self . step ( InHead , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => false , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => false , _ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => self . unexpected ( & tag ) , ( _ , token ) => {
self . insert_phantom ( local_name ! ( "body" ) ) ; Reprocess ( InBody , token )
}
}
}
} , InBody => match token {
NullCharacterToken => self . unexpected ( & token ) , CharacterTokens ( _ , text ) => {
self . reconstruct_formatting ( ) ; if any_not_whitespace ( & text ) {
self . frameset_ok = false ;
} self . append_text ( text )
} CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => {
self . unexpected ( & tag ) ; if ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
let top = html_elem ( & self . open_elems ) ; self . sink . add_attrs_if_missing ( top , tag . attrs ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "base" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "basefont" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "bgsound" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "link" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "title" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => {
self . step ( InHead , token )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "body" ) , ..
} ) => {
self . unexpected ( & tag ) ; match self . body_elem ( ) . cloned ( ) {
Some ( ref node ) if self . open_elems . len ( ) != 1 && ! self . in_html_elem_named ( local_name ! ( "template" ) ) => {
self . frameset_ok = false ; self . sink . add_attrs_if_missing ( node , tag . attrs )
} , _ => {
}
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "frameset" ) , ..
} ) => {
self . unexpected ( & tag ) ; if ! self . frameset_ok {
return Done ;
} let body = unwrap_or_return ! ( self . body_elem ( ) , Done ) . clone ( ) ; self . sink . remove_from_parent ( & body ) ; self . open_elems . truncate ( 1 ) ; self . insert_element_for ( tag ) ; self . mode = InFrameset ; Done
} EOFToken => {
if ! self . template_modes . is_empty ( ) {
self . step ( InTemplate , token )
} else {
self . check_body_end ( ) ; self . stop_parsing ( )
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) => {
if self . in_scope_named ( default_scope , local_name ! ( "body" ) ) {
self . check_body_end ( ) ; self . mode = AfterBody ;
} else {
self . sink . parse_error ( Borrowed ( "</body> with no <body> in scope" ) ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => {
if self . in_scope_named ( default_scope , local_name ! ( "body" ) ) {
self . check_body_end ( ) ; Reprocess ( AfterBody , token )
} else {
self . sink . parse_error ( Borrowed ( "</html> with no <body> in scope" ) ) ; Done
}
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "address" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "article" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "aside" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "blockquote" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "center" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "details" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dialog" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dir" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "div" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dl" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "fieldset" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "figcaption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "figure" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "footer" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "header" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "hgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "main" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "nav" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "ol" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "p" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "section" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "summary" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "ul" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "menu" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h1" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h2" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h3" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h4" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h5" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h6" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; if self . current_node_in ( heading_tag ) {
self . sink . parse_error ( Borrowed ( "nested heading tags" ) ) ; self . pop ( ) ;
} self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "pre" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "listing" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . insert_element_for ( tag ) ; self . ignore_lf = true ; self . frameset_ok = false ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "form" ) , ..
} ) => {
if self . form_elem . is_some ( ) && ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
self . sink . parse_error ( Borrowed ( "nested forms" ) ) ;
} else {
self . close_p_element_in_button_scope ( ) ; let elem = self . insert_element_for ( tag ) ; if ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
self . form_elem = Some ( elem ) ;
}
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "li" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dd" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dt" ) , ..
} ) => {
declare_tag_set ! ( close_list = "li" ) ; declare_tag_set ! ( close_defn = "dd" "dt" ) ; declare_tag_set ! ( extra_special = [ special_tag ] - "address" "div" "p" ) ; let list = match tag . name {
local_name ! ( "li" ) => true , local_name ! ( "dd" ) | local_name ! ( "dt" ) => false , _ => unreachable ! ( ) ,
} ; self . frameset_ok = false ; let mut to_close = None ; for node in self . open_elems . iter ( ) . rev ( ) {
let name = self . sink . elem_name ( node ) ; let can_close = if list {
close_list ( name )
} else {
close_defn ( name )
} ; if can_close {
to_close = Some ( name . local . clone ( ) ) ; break ;
} if extra_special ( name ) {
break ;
}
} match to_close {
Some ( name ) => {
self . generate_implied_end_except ( name . clone ( ) ) ; self . expect_to_close ( name ) ;
} None => ( ) ,
} self . close_p_element_in_button_scope ( ) ; self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "plaintext" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . insert_element_for ( tag ) ; ToPlaintext
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "button" ) , ..
} ) => {
if self . in_scope_named ( default_scope , local_name ! ( "button" ) ) {
self . sink . parse_error ( Borrowed ( "nested buttons" ) ) ; self . generate_implied_end ( cursory_implied_end ) ; self . pop_until_named ( local_name ! ( "button" ) ) ;
} self . reconstruct_formatting ( ) ; self . insert_element_for ( tag ) ; self . frameset_ok = false ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "address" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "article" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "aside" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "blockquote" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "button" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "center" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "details" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "dialog" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "dir" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "div" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "dl" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "fieldset" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "figcaption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "figure" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "footer" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "header" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "hgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "listing" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "main" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "menu" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "nav" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "ol" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "pre" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "section" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "summary" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "ul" ) , ..
} ) => {
if ! self . in_scope_named ( default_scope , tag . name . clone ( ) ) {
self . unexpected ( & tag ) ;
} else {
self . generate_implied_end ( cursory_implied_end ) ; self . expect_to_close ( tag . name ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "form" ) , ..
} ) => {
if ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
let node = match self . form_elem . take ( ) {
None => {
self . sink . parse_error ( Borrowed ( "Null form element pointer on </form>" ) ) ; return Done ;
} Some ( x ) => x ,
} ; if ! self . in_scope ( default_scope , | n | self . sink . same_node ( & node , & n ) ) {
self . sink . parse_error ( Borrowed ( "Form element not in scope on </form>" ) ) ; return Done ;
} self . generate_implied_end ( cursory_implied_end ) ; let current = self . current_node ( ) . clone ( ) ; self . remove_from_stack ( & node ) ; if ! self . sink . same_node ( & current , & node ) {
self . sink . parse_error ( Borrowed ( "Bad open element on </form>" ) ) ;
}
} else {
if ! self . in_scope_named ( default_scope , local_name ! ( "form" ) ) {
self . sink . parse_error ( Borrowed ( "Form element not in scope on </form>" ) ) ; return Done ;
} self . generate_implied_end ( cursory_implied_end ) ; if ! self . current_node_named ( local_name ! ( "form" ) ) {
self . sink . parse_error ( Borrowed ( "Bad open element on </form>" ) ) ;
} self . pop_until_named ( local_name ! ( "form" ) ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "p" ) , ..
} ) => {
if ! self . in_scope_named ( button_scope , local_name ! ( "p" ) ) {
self . sink . parse_error ( Borrowed ( "No <p> tag to close" ) ) ; self . insert_phantom ( local_name ! ( "p" ) ) ;
} self . close_p_element ( ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "li" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "dd" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "dt" ) , ..
} ) => {
let in_scope = if tag . name == local_name ! ( "li" ) {
self . in_scope_named ( list_item_scope , tag . name . clone ( ) )
} else {
self . in_scope_named ( default_scope , tag . name . clone ( ) )
} ; if in_scope {
self . generate_implied_end_except ( tag . name . clone ( ) ) ; self . expect_to_close ( tag . name ) ;
} else {
self . sink . parse_error ( Borrowed ( "No matching tag to close" ) ) ;
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h1" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h2" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h3" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h4" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h5" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "h6" ) , ..
} ) => {
if self . in_scope ( default_scope , | n | self . elem_in ( & n , heading_tag ) ) {
self . generate_implied_end ( cursory_implied_end ) ; if ! self . current_node_named ( tag . name ) {
self . sink . parse_error ( Borrowed ( "Closing wrong heading tag" ) ) ;
} self . pop_until ( heading_tag ) ;
} else {
self . sink . parse_error ( Borrowed ( "No heading tag to close" ) ) ;
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "a" ) , ..
} ) => {
self . handle_misnested_a_tags ( & tag ) ; self . reconstruct_formatting ( ) ; self . create_formatting_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "b" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "big" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "code" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "em" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "font" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "i" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "s" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "small" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "strike" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "strong" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tt" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "u" ) , ..
} ) => {
self . reconstruct_formatting ( ) ; self . create_formatting_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "nobr" ) , ..
} ) => {
self . reconstruct_formatting ( ) ; if self . in_scope_named ( default_scope , local_name ! ( "nobr" ) ) {
self . sink . parse_error ( Borrowed ( "Nested <nobr>" ) ) ; self . adoption_agency ( local_name ! ( "nobr" ) ) ; self . reconstruct_formatting ( ) ;
} self . create_formatting_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "a" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "b" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "big" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "code" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "em" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "font" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "i" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "nobr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "s" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "small" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "strike" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "strong" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tt" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "u" ) , ..
} ) => {
self . adoption_agency ( tag . name ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "applet" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "marquee" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "object" ) , ..
} ) => {
self . reconstruct_formatting ( ) ; self . insert_element_for ( tag ) ; self . active_formatting . push ( Marker ) ; self . frameset_ok = false ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "applet" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "marquee" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "object" ) , ..
} ) => {
if ! self . in_scope_named ( default_scope , tag . name . clone ( ) ) {
self . unexpected ( & tag ) ;
} else {
self . generate_implied_end ( cursory_implied_end ) ; self . expect_to_close ( tag . name ) ; self . clear_active_formatting_to_marker ( ) ;
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "table" ) , ..
} ) => {
if self . quirks_mode != Quirks {
self . close_p_element_in_button_scope ( ) ;
} self . insert_element_for ( tag ) ; self . frameset_ok = false ; self . mode = InTable ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "br" ) , ..
} ) => {
self . unexpected ( & tag ) ; self . step ( InBody , TagToken ( Tag {
kind : StartTag , attrs : vec ! ( ) , .. tag
} ) )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "area" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "br" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "embed" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "img" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "keygen" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "wbr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "input" ) , ..
} ) => {
let keep_frameset_ok = match tag . name {
local_name ! ( "input" ) => self . is_type_hidden ( & tag ) , _ => false ,
} ; self . reconstruct_formatting ( ) ; self . insert_and_pop_element_for ( tag ) ; if ! keep_frameset_ok {
self . frameset_ok = false ;
} DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "param" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "source" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "track" ) , ..
} ) => {
self . insert_and_pop_element_for ( tag ) ; DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "hr" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . insert_and_pop_element_for ( tag ) ; self . frameset_ok = false ; DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "image" ) , ..
} ) => {
self . unexpected ( & tag ) ; self . step ( InBody , TagToken ( Tag {
name : local_name ! ( "img" ) , .. tag
} ) )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "textarea" ) , ..
} ) => {
self . ignore_lf = true ; self . frameset_ok = false ; self . parse_raw_data ( tag , Rcdata )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "xmp" ) , ..
} ) => {
self . close_p_element_in_button_scope ( ) ; self . reconstruct_formatting ( ) ; self . frameset_ok = false ; self . parse_raw_data ( tag , Rawtext )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "iframe" ) , ..
} ) => {
self . frameset_ok = false ; self . parse_raw_data ( tag , Rawtext )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noembed" ) , ..
} ) => {
self . parse_raw_data ( tag , Rawtext )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "select" ) , ..
} ) => {
self . reconstruct_formatting ( ) ; self . insert_element_for ( tag ) ; self . frameset_ok = false ; self . mode = match self . mode {
InTable | InCaption | InTableBody | InRow | InCell => InSelectInTable , _ => InSelect ,
} ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "optgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "option" ) , ..
} ) => {
if self . current_node_named ( local_name ! ( "option" ) ) {
self . pop ( ) ;
} self . reconstruct_formatting ( ) ; self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "rb" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "rtc" ) , ..
} ) => {
if self . in_scope_named ( default_scope , local_name ! ( "ruby" ) ) {
self . generate_implied_end ( cursory_implied_end ) ;
} if ! self . current_node_named ( local_name ! ( "ruby" ) ) {
self . unexpected ( & tag ) ;
} self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "rp" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "rt" ) , ..
} ) => {
if self . in_scope_named ( default_scope , local_name ! ( "ruby" ) ) {
self . generate_implied_end_except ( local_name ! ( "rtc" ) ) ;
} if ! self . current_node_named ( local_name ! ( "rtc" ) ) && ! self . current_node_named ( local_name ! ( "ruby" ) ) {
self . unexpected ( & tag ) ;
} self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "math" ) , ..
} ) => self . enter_foreign ( tag , ns ! ( mathml ) ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "svg" ) , ..
} ) => self . enter_foreign ( tag , ns ! ( svg ) ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "frame" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) => {
self . unexpected ( & token ) ; Done
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , ..
} ) ) => {
if self . opts . scripting_enabled && tag . name == local_name ! ( "noscript" ) {
self . parse_raw_data ( tag , Rawtext )
} else {
self . reconstruct_formatting ( ) ; self . insert_element_for ( tag ) ; Done
}
} ( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => {
self . process_end_tag_in_body ( tag ) ; Done
} ( _ , _ ) => panic ! ( "impossible case in InBody mode" ) ,
}
}
} , Text => match token {
CharacterTokens ( _ , text ) => self . append_text ( text ) , EOFToken => {
self . unexpected ( & token ) ; if self . current_node_named ( local_name ! ( "script" ) ) {
let current = current_node ( & self . open_elems ) ; self . sink . mark_script_already_started ( current ) ;
} self . pop ( ) ; Reprocess ( self . orig_mode . take ( ) . unwrap ( ) , token )
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => {
let node = self . pop ( ) ; self . mode = self . orig_mode . take ( ) . unwrap ( ) ; if tag . name == local_name ! ( "script" ) {
return Script ( node ) ;
} Done
} ( _ , _ ) => panic ! ( "impossible case in Text mode" ) ,
}
}
} , InTable => match token {
NullCharacterToken => self . process_chars_in_table ( token ) , CharacterTokens ( .. ) => self . process_chars_in_table ( token ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) => {
self . pop_until_current ( table_scope ) ; self . active_formatting . push ( Marker ) ; self . insert_element_for ( tag ) ; self . mode = InCaption ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) => {
self . pop_until_current ( table_scope ) ; self . insert_element_for ( tag ) ; self . mode = InColumnGroup ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) => {
self . pop_until_current ( table_scope ) ; self . insert_phantom ( local_name ! ( "colgroup" ) ) ; Reprocess ( InColumnGroup , token )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) => {
self . pop_until_current ( table_scope ) ; self . insert_element_for ( tag ) ; self . mode = InTableBody ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) => {
self . pop_until_current ( table_scope ) ; self . insert_phantom ( local_name ! ( "tbody" ) ) ; Reprocess ( InTableBody , token )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "table" ) , ..
} ) => {
self . unexpected ( & token ) ; if self . in_scope_named ( table_scope , local_name ! ( "table" ) ) {
self . pop_until_named ( local_name ! ( "table" ) ) ; Reprocess ( self . reset_insertion_mode ( ) , token )
} else {
Done
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) => {
if self . in_scope_named ( table_scope , local_name ! ( "table" ) ) {
self . pop_until_named ( local_name ! ( "table" ) ) ; self . mode = self . reset_insertion_mode ( ) ;
} else {
self . unexpected ( & token ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) => self . unexpected ( & token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => self . step ( InHead , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "input" ) , ..
} ) => {
self . unexpected ( & tag ) ; if self . is_type_hidden ( & tag ) {
self . insert_and_pop_element_for ( tag ) ; DoneAckSelfClosing
} else {
self . foster_parent_in_body ( TagToken ( tag ) )
}
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "form" ) , ..
} ) => {
self . unexpected ( & tag ) ; if ! self . in_html_elem_named ( local_name ! ( "template" ) ) && self . form_elem . is_none ( ) {
self . form_elem = Some ( self . insert_and_pop_element_for ( tag ) ) ;
} Done
} EOFToken => self . step ( InBody , token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
self . unexpected ( & token ) ; self . foster_parent_in_body ( token )
}
}
}
} , InTableText => match token {
NullCharacterToken => self . unexpected ( & token ) , CharacterTokens ( split , text ) => {
self . pending_table_text . push ( ( split , text ) ) ; Done
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
let pending = replace ( & mut self . pending_table_text , vec ! ( ) ) ; let contains_nonspace = pending . iter ( ) . any ( | & ( split , ref text ) | {
match split {
Whitespace => false , NotWhitespace => true , NotSplit => any_not_whitespace ( text ) ,
}
} ) ; if contains_nonspace {
self . sink . parse_error ( Borrowed ( "Non-space table text" ) ) ; for ( split , text ) in pending . into_iter ( ) {
match self . foster_parent_in_body ( CharacterTokens ( split , text ) ) {
Done => ( ) , _ => panic ! ( "not prepared to handle this!" ) ,
}
}
} else {
for ( _ , text ) in pending . into_iter ( ) {
self . append_text ( text ) ;
}
} Reprocess ( self . orig_mode . take ( ) . unwrap ( ) , token )
}
}
}
} , InCaption => match token {
:: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) => {
if self . in_scope_named ( table_scope , local_name ! ( "caption" ) ) {
self . generate_implied_end ( cursory_implied_end ) ; self . expect_to_close ( local_name ! ( "caption" ) ) ; self . clear_active_formatting_to_marker ( ) ; match tag {
Tag {
kind : EndTag , name : local_name ! ( "caption" ) , ..
} => {
self . mode = InTable ; Done
} _ => Reprocess ( InTable , TagToken ( tag ) )
}
} else {
self . unexpected ( & tag ) ; Done
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . step ( InBody , token ) ,
}
}
} , InColumnGroup => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) => {
self . insert_and_pop_element_for ( tag ) ; DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) => {
if self . current_node_named ( local_name ! ( "colgroup" ) ) {
self . pop ( ) ; self . mode = InTable ;
} else {
self . unexpected ( & token ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) => self . unexpected ( & token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => self . step ( InHead , token ) , EOFToken => self . step ( InBody , token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
if self . current_node_named ( local_name ! ( "colgroup" ) ) {
self . pop ( ) ; Reprocess ( InTable , token )
} else {
self . unexpected ( & token )
}
}
}
}
} , InTableBody => match token {
:: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) => {
self . pop_until_current ( table_body_context ) ; self . insert_element_for ( tag ) ; self . mode = InRow ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) => {
self . unexpected ( & token ) ; self . pop_until_current ( table_body_context ) ; self . insert_phantom ( local_name ! ( "tr" ) ) ; Reprocess ( InRow , token )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) => {
if self . in_scope_named ( table_scope , tag . name . clone ( ) ) {
self . pop_until_current ( table_body_context ) ; self . pop ( ) ; self . mode = InTable ;
} else {
self . unexpected ( & tag ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) => {
declare_tag_set ! ( table_outer = "table" "tbody" "tfoot" ) ; if self . in_scope ( table_scope , | e | self . elem_in ( & e , table_outer ) ) {
self . pop_until_current ( table_body_context ) ; self . pop ( ) ; Reprocess ( InTable , token )
} else {
self . unexpected ( & token )
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . step ( InTable , token ) ,
}
}
} , InRow => match token {
:: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) => {
self . pop_until_current ( table_row_context ) ; self . insert_element_for ( tag ) ; self . mode = InCell ; self . active_formatting . push ( Marker ) ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) => {
if self . in_scope_named ( table_scope , local_name ! ( "tr" ) ) {
self . pop_until_current ( table_row_context ) ; let node = self . pop ( ) ; self . assert_named ( & node , local_name ! ( "tr" ) ) ; self . mode = InTableBody ;
} else {
self . unexpected ( & token ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) => {
if self . in_scope_named ( table_scope , local_name ! ( "tr" ) ) {
self . pop_until_current ( table_row_context ) ; let node = self . pop ( ) ; self . assert_named ( & node , local_name ! ( "tr" ) ) ; Reprocess ( InTableBody , token )
} else {
self . unexpected ( & token )
}
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) => {
if self . in_scope_named ( table_scope , tag . name . clone ( ) ) {
if self . in_scope_named ( table_scope , local_name ! ( "tr" ) ) {
self . pop_until_current ( table_row_context ) ; let node = self . pop ( ) ; self . assert_named ( & node , local_name ! ( "tr" ) ) ; Reprocess ( InTableBody , TagToken ( tag ) )
} else {
Done
}
} else {
self . unexpected ( & tag )
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) => self . unexpected ( & token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . step ( InTable , token ) ,
}
}
} , InCell => match token {
:: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) => {
if self . in_scope_named ( table_scope , tag . name . clone ( ) ) {
self . generate_implied_end ( cursory_implied_end ) ; self . expect_to_close ( tag . name ) ; self . clear_active_formatting_to_marker ( ) ; self . mode = InRow ;
} else {
self . unexpected ( & tag ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) => {
if self . in_scope ( table_scope , | n | self . elem_in ( & n , td_th ) ) {
self . close_the_cell ( ) ; Reprocess ( InRow , token )
} else {
self . unexpected ( & token )
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "col" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => self . unexpected ( & token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) => {
if self . in_scope_named ( table_scope , tag . name . clone ( ) ) {
self . close_the_cell ( ) ; Reprocess ( InRow , TagToken ( tag ) )
} else {
self . unexpected ( & tag )
}
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . step ( InBody , token ) ,
}
}
} , InSelect => match token {
NullCharacterToken => self . unexpected ( & token ) , CharacterTokens ( _ , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "option" ) , ..
} ) => {
if self . current_node_named ( local_name ! ( "option" ) ) {
self . pop ( ) ;
} self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "optgroup" ) , ..
} ) => {
if self . current_node_named ( local_name ! ( "option" ) ) {
self . pop ( ) ;
} if self . current_node_named ( local_name ! ( "optgroup" ) ) {
self . pop ( ) ;
} self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "optgroup" ) , ..
} ) => {
if self . open_elems . len ( ) >= 2 && self . current_node_named ( local_name ! ( "option" ) ) && self . html_elem_named ( & self . open_elems [ self . open_elems . len ( ) - 2 ] , local_name ! ( "optgroup" ) ) {
self . pop ( ) ;
} if self . current_node_named ( local_name ! ( "optgroup" ) ) {
self . pop ( ) ;
} else {
self . unexpected ( & token ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "option" ) , ..
} ) => {
if self . current_node_named ( local_name ! ( "option" ) ) {
self . pop ( ) ;
} else {
self . unexpected ( & token ) ;
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "select" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "select" ) , ..
} ) => {
let in_scope = self . in_scope_named ( select_scope , local_name ! ( "select" ) ) ; if ! in_scope || tag . kind == StartTag {
self . unexpected ( & tag ) ;
} if in_scope {
self . pop_until_named ( local_name ! ( "select" ) ) ; self . mode = self . reset_insertion_mode ( ) ;
} Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "input" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "keygen" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "textarea" ) , ..
} ) => {
self . unexpected ( & token ) ; if self . in_scope_named ( select_scope , local_name ! ( "select" ) ) {
self . pop_until_named ( local_name ! ( "select" ) ) ; Reprocess ( self . reset_insertion_mode ( ) , token )
} else {
Done
}
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => self . step ( InHead , token ) , EOFToken => self . step ( InBody , token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . unexpected ( & token ) ,
}
}
} , InSelectInTable => match token {
:: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "table" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) => {
self . unexpected ( & token ) ; self . pop_until_named ( local_name ! ( "select" ) ) ; Reprocess ( self . reset_insertion_mode ( ) , token )
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "table" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "thead" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "tr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "th" ) , ..
} ) => {
self . unexpected ( & tag ) ; if self . in_scope_named ( table_scope , tag . name . clone ( ) ) {
self . pop_until_named ( local_name ! ( "select" ) ) ; Reprocess ( self . reset_insertion_mode ( ) , TagToken ( tag ) )
} else {
Done
}
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . step ( InSelect , token ) ,
}
}
} , InTemplate => match token {
CharacterTokens ( _ , _ ) => self . step ( InBody , token ) , CommentToken ( _ ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "base" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "basefont" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "bgsound" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "link" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "script" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "style" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "template" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "title" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "template" ) , ..
} ) => {
self . step ( InHead , token )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "caption" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "colgroup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tbody" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tfoot" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "thead" ) , ..
} ) => {
self . template_modes . pop ( ) ; self . template_modes . push ( InTable ) ; Reprocess ( InTable , token )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "col" ) , ..
} ) => {
self . template_modes . pop ( ) ; self . template_modes . push ( InColumnGroup ) ; Reprocess ( InColumnGroup , token )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tr" ) , ..
} ) => {
self . template_modes . pop ( ) ; self . template_modes . push ( InTableBody ) ; Reprocess ( InTableBody , token )
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "td" ) , ..
} ) | :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "th" ) , ..
} ) => {
self . template_modes . pop ( ) ; self . template_modes . push ( InRow ) ; Reprocess ( InRow , token )
} EOFToken => {
if ! self . in_html_elem_named ( local_name ! ( "template" ) ) {
self . stop_parsing ( )
} else {
self . unexpected ( & token ) ; self . pop_until_named ( local_name ! ( "template" ) ) ; self . clear_active_formatting_to_marker ( ) ; self . template_modes . pop ( ) ; self . mode = self . reset_insertion_mode ( ) ; Reprocess ( self . reset_insertion_mode ( ) , token )
}
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , ..
} ) ) => {
self . template_modes . pop ( ) ; self . template_modes . push ( InBody ) ; Reprocess ( InBody , TagToken ( tag ) )
} ( _ , token ) => self . unexpected ( & token ) ,
}
}
} , AfterBody => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => self . step ( InBody , token ) , CommentToken ( text ) => self . append_comment_to_html ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => {
if self . is_fragment ( ) {
self . unexpected ( & token ) ;
} else {
self . mode = AfterAfterBody ;
} Done
} EOFToken => self . stop_parsing ( ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
self . unexpected ( & token ) ; Reprocess ( InBody , token )
}
}
}
} , InFrameset => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "frameset" ) , ..
} ) => {
self . insert_element_for ( tag ) ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "frameset" ) , ..
} ) => {
if self . open_elems . len ( ) == 1 {
self . unexpected ( & token ) ;
} else {
self . pop ( ) ; if ! self . is_fragment ( ) && ! self . current_node_named ( local_name ! ( "frameset" ) ) {
self . mode = AfterFrameset ;
}
} Done
} :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "frame" ) , ..
} ) => {
self . insert_and_pop_element_for ( tag ) ; DoneAckSelfClosing
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) => self . step ( InHead , token ) , EOFToken => {
if self . open_elems . len ( ) != 1 {
self . unexpected ( & token ) ;
} self . stop_parsing ( )
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . unexpected ( & token ) ,
}
}
} , AfterFrameset => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , text ) => self . append_text ( text ) , CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , name : local_name ! ( "html" ) , ..
} ) => {
self . mode = AfterAfterFrameset ; Done
} :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) => self . step ( InHead , token ) , EOFToken => self . stop_parsing ( ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . unexpected ( & token ) ,
}
}
} , AfterAfterBody => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => self . step ( InBody , token ) , CommentToken ( text ) => self . append_comment_to_doc ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , EOFToken => self . stop_parsing ( ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => {
self . unexpected ( & token ) ; Reprocess ( InBody , token )
}
}
}
} , AfterAfterFrameset => match token {
CharacterTokens ( NotSplit , text ) => SplitWhitespace ( text ) , CharacterTokens ( Whitespace , _ ) => self . step ( InBody , token ) , CommentToken ( text ) => self . append_comment_to_doc ( text ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "html" ) , ..
} ) => self . step ( InBody , token ) , EOFToken => self . stop_parsing ( ) , :: tree_builder :: types :: TagToken ( :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "noframes" ) , ..
} ) => self . step ( InHead , token ) , last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( _ , token ) => self . unexpected ( & token ) ,
}
}
} ,
}
} fn step_foreign ( & mut self , token : Token ) -> ProcessResult < Handle > {
match token {
NullCharacterToken => {
self . unexpected ( & token ) ; self . append_text ( "\u{fffd}" . to_tendril ( ) )
} CharacterTokens ( _ , text ) => {
if any_not_whitespace ( & text ) {
self . frameset_ok = false ;
} self . append_text ( text )
} CommentToken ( text ) => self . append_comment ( text ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "b" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "big" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "blockquote" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "body" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "br" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "center" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "code" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dd" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "div" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dl" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "dt" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "em" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "embed" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h1" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h2" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h3" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h4" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h5" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "h6" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "head" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "hr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "i" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "img" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "li" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "listing" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "menu" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "meta" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "nobr" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "ol" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "p" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "pre" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "ruby" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "s" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "small" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "span" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "strong" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "strike" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "sub" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "sup" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "table" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "tt" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "u" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "ul" ) , ..
} ) | :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "var" ) , ..
} ) => self . unexpected_start_tag_in_foreign_content ( tag ) , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , name : local_name ! ( "font" ) , ..
} ) => {
let unexpected = tag . attrs . iter ( ) . any ( | attr | {
matches ! ( attr . name . expanded ( ) , expanded_name ! ( "" , "color" ) | expanded_name ! ( "" , "face" ) | expanded_name ! ( "" , "size" ) )
} ) ; if unexpected {
self . unexpected_start_tag_in_foreign_content ( tag )
} else {
self . foreign_start_tag ( tag )
}
} last_arm_token => {
let enable_wildcards = match last_arm_token {
_ => true ,
} ; match ( enable_wildcards , last_arm_token ) {
( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: StartTag , ..
} ) ) => self . foreign_start_tag ( tag ) , ( true , :: tree_builder :: types :: TagToken ( tag @ :: tokenizer :: Tag {
kind : :: tokenizer :: EndTag , ..
} ) ) => {
let mut first = true ; let mut stack_idx = self . open_elems . len ( ) - 1 ; loop {
if stack_idx == 0 {
return Done ;
} let html ; let eq ; {
let node_name = self . sink . elem_name ( & self . open_elems [ stack_idx ] ) ; html = * node_name . ns == ns ! ( html ) ; eq = node_name . local . eq_ignore_ascii_case ( & tag . name ) ;
} if ! first && html {
let mode = self . mode ; return self . step ( mode , TagToken ( tag ) ) ;
} if eq {
self . open_elems . truncate ( stack_idx ) ; return Done ;
} if first {
self . unexpected ( & tag ) ; first = false ;
} stack_idx -= 1 ;
}
} ( _ , _ ) => panic ! ( "impossible case in foreign content" ) ,
}
}
}
}
}