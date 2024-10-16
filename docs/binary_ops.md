# Add
| LHS/RHS |  None  | TRUE |  2   |  xyz   | List |  Map   | Object | Error |
|:-------:|:------:|:----:|:----:|:------:|:----:|:------:|:------:|:-----:| 
|  None   |  None  | TRUE |  2   |  xyz   | List |  Map   | Object |   X   | 
|  FALSE  | FALSE  | TRUE |  2   |   X    |  X   |   X    |   X    |   X   |
|    1    |   1    |  2   |  3   |   X    |  X   |   X    |   X    |   X   |
|   abc   |  abc   |  X   |  X   | abcxyz |  X   |   X    |   X    |   X   |
|  List   |  List  | List | List |  List  | List |  List  |  List  |   X   |
|   Map   |  Map   | Map  | Map  |  Map   |  X   |  Map   | Object |   X   |
| Object  | Object |  X   |  X   |   X    |  X   | Object | Object |   X   |
|  Error  |   X    |  X   |  X   |   X    |  X   |   X    |   X    |   X   |

# Mul
| LHS/RHS | None | TRUE  |   2    |  xyz   |  List  | Map  | Object | Error |
|:-------:|:----:|:-----:|:------:|:------:|:------:|:----:|:------:|:-----:|
|   Mul   | None | TRUE  |   2    |  xyz   |  List  | Map  | Object | Error |
|  None   | None | None  |  None  |  None  |  None  | None |  None  |   X   |
|  FALSE  | None | FALSE | FALSE  |   X    |   X    |  X   |   X    |   X   |
|    1    | None |   1   |   2    |   X    |   X    |  X   |   X    |   X   |
|   abc   | None |   X   | abcabc |   X    | String |  X   |   X    |   X   |
|  List   | None | List  |  List  | String |   X    |  X   |   X    |   X   |
|   Map   | None |  Map  |  List  |   X    |   X    |  X   |   X    |   X   |
| Object  | None |   X   |   X    |   X    |   X    |  X   |   X    |   X   |
|  Error  |  X   |   X   |   X    |   X    |   X    |  X   |   X    |   X   |

# Shl
| LHS/RHS |  None  | TRUE  |  2   |  xyz   | List |  Map   | Object | Error |
|:-------:|:------:|:-----:|:----:|:------:|:----:|:------:|:------:|:-----:|
|         |  None  | TRUE  |  2   |  xyz   | List |  Map   | Object | Error |
|  None   |  None  | None  |  2   |  xyz   | List |  Map   | Object |   X   |
|  FALSE  | FALSE  | FALSE |  2   |   X    |  X   |   X    |   X    |   X   |
|    1    |   1    |   2   |  3   |   X    |  X   |   X    |   X    |   X   |
|   abc   |  abc   |   X   |  c   | xyzabc |  X   |   X    |   X    |   X   |
|  List   |  List  | List  | List |  List  | List |  List  |  List  |   X   |
|   Map   |  Map   |  Map  | Map  |  Map   |  X   |  Map   | Object |   X   |
| Object  | Object |   X   |  X   |   X    |  X   | Object | Object |   X   |
|  Error  |   X    |   X   |  X   |   X    |  X   |   X    |   X    |   X   |

# And
| LHS/RHS | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object | Error |
|:-------:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:------:|:-----:|
|   And   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object | Error |
|  None   | None  | None  | None  | None  | None  | None  |  None  |   X   |
|  FALSE  | FALSE | FALSE | FALSE | FALSE | FALSE | FALSE | FALSE  |   X   |
|    1    | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|   abc   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|  List   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|   Map   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
| Object  | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|  Error  |   X   |   X   |   X   |   X   |   X   |   X   |   X    |   X   |

# Xor
| LHS/RHS |  None  | TRUE |  2   | xyz  | List | Map  | Object | Error |
|:-------:|:------:|:----:|:----:|:----:|:----:|:----:|:------:|:-----:|
|  None   |  None  | TRUE |  2   | xyz  | List | Map  | Object |   X   |
|  FALSE  |  None  | TRUE |  2   | xyz  | List | Map  | Object |   X   |
|    1    |   1    | None | None | None | None | None |  None  |   X   |
|   abc   |  abc   | None | None | None | None | None |  None  |   X   |
|  List   |  List  | None | None | None | None | None |  None  |   X   |
|   Map   |  Map   | None | None | None | None | None |  None  |   X   |
| Object  | Object | None | None | None | None | None |  None  |   X   |
|  Error  |   X    |  X   |  X   |  X   |  X   |  X   |   X    |   X   |

# BitOr
| LHS/RHS |  None  | TRUE |  2   |  xyz   | List |  Map   | Object | Error |
|:-------:|:------:|:----:|:----:|:------:|:----:|:------:|:------:|:-----:|
|         |  None  | TRUE |  2   |  xyz   | List |  Map   | Object | Error |
|  None   |  None  | TRUE |  2   |  xyz   | List |  Map   | Object |   X   |
|  FALSE  | FALSE  | TRUE |  2   |   X    |  X   |   X    |   X    |   X   |
|    1    |   1    |  2   |  3   |   X    |  X   |   X    |   X    |   X   |
|   abc   |  abc   |  X   |  X   | abcxyz |  X   |   X    |   X    |   X   |
|  List   |  List  | List | List |  List  | List |  List  |  List  |   X   |
|   Map   |  Map   | Map  | Map  |  Map   |  X   |  Map   | Object |   X   |
| Object  | Object |  X   |  X   |   X    |  X   | Object | Object |   X   |
|  Error  |   X    |  X   |  X   |   X    |  X   |   X    |   X    |   X   |

# Sub
| LHS/RHS |  None  | TRUE |  2   | xyz  | List |  Map   | Object | Error |
|:-------:|:------:|:----:|:----:|:----:|:----:|:------:|:------:|:-----:|
|  None   |  None  | TRUE |  -2  | xyz  | List |  Map   | Object |   X   |
|  FALSE  | FALSE  | TRUE |  -2  |  X   |  X   |   X    |   X    |   X   |
|    1    |   1    |  2   |  -1  |  X   |  X   |   X    |   X    |   X   |
|   abc   |  abc   |  X   |  X   | abc  |  X   |   X    |   X    |   X   |
|  List   |  List  | List | List | List | List |  List  |  List  |   X   |
|   Map   |  Map   | Map  | Map  | Map  |  X   |  Map   | Object |   X   |
| Object  | Object |  X   |  X   |  X   |  X   | Object | Object |   X   |
|  Error  |   X    |  X   |  X   |  X   |  X   |   X    |   X    |   X   |

# Div
| LHS/RHS | None | TRUE  |   2   |     xyz      | List | Map  | Object | Error |
|:-------:|:----:|:-----:|:-----:|:------------:|:----:|:----:|:------:|:-----:|
|  None   |  X   | None  | None  |     None     | None | None |  None  |   X   |
|  FALSE  |  X   | FALSE | FALSE |      X       |  X   |  X   |   X    |   X   |
|    1    |  X   |   1   |   0   |      X       |  X   |  X   |   X    |   X   |
|   abc   |  X   |   X   |   X   | List (split) |  X   |  X   |   X    |   X   |
|  List   |  X   | List  |   X   |      X       |  X   |  X   |   X    |   X   |
|   Map   |  X   |  Map  |   X   |      X       |  X   |  X   |   X    |   X   |
| Object  |  X   |   X   |   X   |      X       |  X   |  X   |   X    |   X   |
|  Error  |  X   |   X   |   X   |      X       |  X   |  X   |   X    |   X   |

# Shr
| LHS/RHS |  None  | TRUE  |  2   |  xyz   | List |  Map   | Object | Error |
|:-------:|:------:|:-----:|:----:|:------:|:----:|:------:|:------:|:-----:|
|  None   |  None  | TRUE  |  2   |  xyz   | List |  Map   | Object |   X   |
|  FALSE  | FALSE  | FALSE |  2   |   X    |  X   |   X    |   X    |   X   |
|    1    |   1    |   0   |  0   |   X    |  X   |   X    |   X    |   X   |
|   abc   |  abc   |   X   |  X   | abcxyz |  X   |   X    |   X    |   X   |
|  List   |  List  | List  | List |  List  | List |  List  |  List  |   X   |
|   Map   |  Map   |  Map  | Map  |  Map   |  X   |  Map   | Object |   X   |
| Object  | Object |   X   |  X   |   X    |  X   | Object | Object |   X   |
|  Error  |   X    |   X   |  X   |   X    |  X   |   X    |   X    |   X   |

# Or						
| LHS/RHS |  None  |  TRUE  |   2    |  xyz   |  List  |  Map   | Object | Error |
|:-------:|:------:|:------:|:------:|:------:|:------:|:------:|:------:|:-----:|
|  None   |  None  |  TRUE  |   2    |  xyz   |  List  |  Map   | Object |   X   |
|  FALSE  |  None  |  TRUE  |   2    |  xyz   |  List  |  Map   | Object |   X   |
|    1    |   1    |   1    |   1    |   1    |   1    |   1    |   1    |   X   |
|   abc   |  abc   |  abc   |  abc   |  abc   |  abc   |  abc   |  abc   |   X   |
|  List   |  List  |  List  |  List  |  List  |  List  |  List  |  List  |   X   |
|   Map   |  Map   |  Map   |  Map   |  Map   |  Map   |  Map   | Object |   X   |
| Object  | Object | Object | Object | Object | Object | Object | Object |   X   |
|  Error  |   X    |   X    |   X    |   X    |   X    |   X    |   X    |   X   |

# BitAnd
| LHS/RHS | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object | Error |
|:-------:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:------:|:-----:|
|  None   | None  | None  | None  | None  | None  | None  |  None  |   X   |
|  FALSE  | FALSE | FALSE | FALSE | FALSE | FALSE | FALSE | FALSE  |   X   |
|    1    | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|   abc   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|  List   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|   Map   | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
| Object  | None  | TRUE  |   2   |  xyz  | List  |  Map  | Object |   X   |
|  Error  |   X   |   X   |   X   |   X   |   X   |   X   |   X    |   X   |

# BitXor
| LHS/RHS |  None  | TRUE |  2   |  xyz   | List |  Map   | Object | Error |
|:-------:|:------:|:----:|:----:|:------:|:----:|:------:|:------:|:-----:|
|  None   |  None  | TRUE |  2   |  xyz   | List |  Map   | Object |   X   |
|  FALSE  | FALSE  | TRUE |  2   |   X    |  X   |   X    |   X    |   X   |
|    1    |   1    |  2   |  3   |   X    |  X   |   X    |   X    |   X   |
|   abc   |  abc   |  X   |  X   | abcxyz |  X   |   X    |   X    |   X   |
|  List   |  List  | List | List |  List  | List |  List  |  List  |   X   |
|   Map   |  Map   | Map  | Map  |  Map   |  X   |  Map   | Object |   X   |
| Object  | Object |  X   |  X   |   X    |  X   | Object | Object |   X   |
|  Error  |   X    |  X   |  X   |   X    |  X   |   X    |   X    |   X   |