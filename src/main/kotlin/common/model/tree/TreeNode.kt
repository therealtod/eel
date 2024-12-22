package eelst.ilike.common.model.tree

interface TreeNode<T> {
    val data: T

    fun getChildren(): Collection<TreeNode<T>>

    /**
     * Add a child to this node
     */
    fun addChild(child: TreeNode<T>)

    fun isLeaf(): Boolean
}
