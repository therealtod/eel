package testcommon

import eelst.ilike.common.Utils

object TestUtils {
    fun loadHanabLivePayload(payloadName: String): String {
        return Utils.getResourceFileContentAsString("hanablive/payloads/$payloadName")
    }
}
