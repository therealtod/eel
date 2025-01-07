package eelst.ilike.common.datastructure.tree

/**
 * A Node of a [Tree] structure
 */
interface TreeNode<T> {
    val data: T

    /**
     * @return a [Collection] of children of this node. If this is a leaf node it should return an empty [Collection]
     */
    fun getChildren(): Collection<TreeNode<T>>

    /**
     * Add a child to this node
     */
    fun addChild(child: TreeNode<T>)

    /**
     * @return a [Boolean] which is true if this is a leaf node
     */
    fun isLeaf(): Boolean
}
