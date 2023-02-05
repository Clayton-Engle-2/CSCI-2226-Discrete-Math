
# These functions all perform set operations by returning the API

def set_union(A, B):
    return A.union(B)

def set_intersection(A, B):
    return A.intersection(B)

def set_difference(A, B):
    return A.difference(B)

def set_isSubset(A, B):
    return A.issubset(B)

# This function checks if A is a proper subset of B,
# by checking if A is a subset of B, and that A and B are not equal
def set_isProperSubset(A, B):
    return A.issubset(B) and A != B



def Test01():
    A = {1, 2, 8}
    B = {1, 2, 3, 4, 5}
    print(set_union(A,B))
    print(set_intersection(A,B)) 
    print(set_difference(A, B))
    print(set_difference(B, A))
    print(set_isSubset(A, B))
    print(set_isProperSubset(A,B)) 

# Console output after running the Test01 function
#   {1, 2, 3, 4, 5, 8}
#   {1, 2}
#   {8}
#   {3, 4, 5}
#   True
#   False