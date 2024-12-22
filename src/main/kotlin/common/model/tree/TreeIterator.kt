package eelst.ilike.common.model.tree

class TreeIterator<T>(root: TreeNode<T>) : Iterator<TreeNode<T>> {
    private val stack: MutableList<Iterator<TreeNode<T>>> = mutableListOf()

    init {
        stack.add(listOf(root).iterator())  // Start with the root node
    }

    override fun hasNext(): Boolean {
        // Check if there's a valid iterator in the stack
        return stack.any { it.hasNext() }
    }

    override fun next(): TreeNode<T> {
        // Find the next available node in the stack
        val currentIterator = stack.first { it.hasNext() }
        val currentNode = currentIterator.next()

        // If the current node has children, add its children iterator to the stack
        if (currentNode.getChildren().isNotEmpty()) {
            stack.add(currentNode.getChildren().iterator())
        }

        return currentNode
    }
}
