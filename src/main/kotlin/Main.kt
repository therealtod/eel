package eelst.ilike

import eelst.ilike.hanablive.HanabLiveConnector
import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.client.HanabLiveHttpClient
import eelst.ilike.utils.Configuration

//TIP To <b>Run</b> code, press <shortcut actionId="Run"/> or
// click the <icon src="AllIcons.Actions.Execute"/> icon in the gutter.
fun main() {
    HanabLiveConnector.establishConnection()



    for (i in 1..5) {
        //TIP Press <shortcut actionId="Debug"/> to start debugging your code. We have set one <icon src="AllIcons.Debugger.Db_set_breakpoint"/> breakpoint
        // for you, but you can always add more by pressing <shortcut actionId="ToggleLineBreakpoint"/>.
        println("i = $i")
    }
}