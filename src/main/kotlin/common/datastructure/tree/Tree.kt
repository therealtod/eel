package eelst.ilike.common.datastructure.tree

/**
 * Tree data structure
 *
 * Can also be considered an acyclic graph.
 */
class Tree<T>(
    private val root: TreeNodeImpl<T>,
    private var traversalOrder: TraversalOrder = TraversalOrder.DEPTH_FIRST
) : Iterable<TreeNode<T>> {

    fun setTraversalOrder(order: TraversalOrder) {
        traversalOrder = order
    }

    override fun iterator(): Iterator<TreeNode<T>> = when (traversalOrder) {
        TraversalOrder.DEPTH_FIRST -> DepthFirstIterator(root)
        TraversalOrder.BREADTH_FIRST -> BreadthFirstIterator(root)
    }

    fun getBranchToLeaf(leaf: TreeNode<T>): List<TreeNode<T>> {
        val path = mutableListOf<TreeNode<T>>()
        if (findPath(root, leaf, path)) {
            return path
        }
        throw IllegalArgumentException("Node $leaf is not a leaf in this tree")
    }

    private fun findPath(
        current: TreeNode<T>,
        target: TreeNode<T>,
        path: MutableList<TreeNode<T>>
    ): Boolean {
        path.add(current)
        if (current == target) {
            return true
        }

        for (child in current.getChildren()) {
            if (findPath(child, target, path)) {
                return true
            }
        }

        path.removeAt(path.size - 1)
        return false
    }

    private class DepthFirstIterator<T>(root: TreeNode<T>) : Iterator<TreeNode<T>> {
        private val stack = ArrayDeque<TreeNode<T>>()

        init {
            stack.add(root)
        }

        override fun hasNext(): Boolean = stack.isNotEmpty()

        override fun next(): TreeNode<T> {
            val current = stack.removeLast()
            stack.addAll(current.getChildren().reversed())
            return current
        }
    }

    private class BreadthFirstIterator<T>(root: TreeNode<T>) : Iterator<TreeNode<T>> {
        private val queue = ArrayDeque<TreeNode<T>>()

        init {
            queue.add(root)
        }

        override fun hasNext(): Boolean = queue.isNotEmpty()

        override fun next(): TreeNode<T> {
            val current = queue.removeFirst()
            queue.addAll(current.getChildren())
            return current
        }
    }
}
