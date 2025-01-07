package common.model.tree

import eelst.ilike.common.datastructure.tree.TraversalOrder
import eelst.ilike.common.datastructure.tree.Tree
import eelst.ilike.common.datastructure.tree.TreeNodeImpl
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class TreeIterationTest {
    @Test
    fun `Should traverse the tree BF`() {
        val root = TreeNodeImpl(1)
        val node2 = TreeNodeImpl(2)
        val node3 = TreeNodeImpl(3)
        val node4 = TreeNodeImpl(4)
        val node5 = TreeNodeImpl(5)
        val node6 = TreeNodeImpl(6)

        // Build the tree structure
        root.addChild(node2)
        root.addChild(node3)
        node2.addChild(node4)
        node2.addChild(node5)
        node3.addChild(node6)
        val tree = Tree(root, traversalOrder = TraversalOrder.BREADTH_FIRST)

        val expected = listOf(root, node2, node3, node4, node5, node6)
        val actual = tree.toList()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should traverse the tree DF`() {
        val root = TreeNodeImpl(1)
        val node2 = TreeNodeImpl(2)
        val node3 = TreeNodeImpl(3)
        val node4 = TreeNodeImpl(4)
        val node5 = TreeNodeImpl(5)
        val node6 = TreeNodeImpl(6)

        // Build the tree structure
        root.addChild(node2)
        root.addChild(node3)
        node2.addChild(node4)
        node2.addChild(node5)
        node3.addChild(node6)
        val tree = Tree(root, traversalOrder = TraversalOrder.DEPTH_FIRST)

        val expected = listOf(root, node2, node4, node5, node3, node6)
        val actual = tree.toList()

        Assertions.assertEquals(expected, actual)
    }
}
