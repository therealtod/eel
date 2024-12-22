package eelst.ilike.common.model.tree

class TreeNodeImpl<T>(override val data: T): TreeNode<T> {
    private val children: MutableList<TreeNode<T>> = mutableListOf()

    override fun getChildren(): Collection<TreeNode<T>> {
        return children
    }

    override fun addChild(child: TreeNode<T>) {
        children.add(child)
    }

    override fun isLeaf(): Boolean {
        return children.isEmpty()
    }

    override fun toString(): String {
        return data.toString()
    }
}
